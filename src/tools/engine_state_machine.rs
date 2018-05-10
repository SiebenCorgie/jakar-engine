use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use jakar_threadpool::ThreadPool;

use core::engine_settings::EngineSettings;

pub enum NextStep {
    ///If a fraem should be rendered
    Render,
    ///If the asset system can be updated
    UpdateAssets,
    ///If the physics system can be updated
    UpdatePhysics,
    ///Tells the system that there is nothing to do. Returns the smallest rest duration. Can be used to wait safely
    Nothing(Duration)
}

pub enum RenderState{
    ///If the renderer does nothing aka, is after the start mode
    Idle,
    ///Is in this state if it is using the asset heavyly. One should try to not use the asset
    /// manager in this time for best performance.
    IsWorkingOnCpu(Instant),
    ///While working mostly on the gpu. The "Instant" marks the time at which the state switched.
    IsWorkingOnGpu(Instant),
}

impl RenderState{
    pub fn work_cpu() -> Self{
        RenderState::IsWorkingOnCpu(Instant::now())
    }

    pub fn work_gpu() -> Self{
        RenderState::IsWorkingOnGpu(Instant::now())
    }

    ///Returns true if `max_speed` is smaller then the time since we wait
    pub fn should_update(&self, max_speed: Duration) -> bool{
        match self{
            &RenderState::IsWorkingOnCpu(_) => false,
            &RenderState::IsWorkingOnGpu(time) => {
                if time.elapsed() > max_speed{
                    true
                }else{
                    false
                }
            }
            &RenderState::Idle => true,
        }
    }

    ///Returns the duration left till we can update next, is Duration(0) if we can update
    pub fn duration_left(&self, max_speed: Duration) -> Duration{
        match self{
            &RenderState::IsWorkingOnGpu(time) => {
                //time since time
                let time_since = time.elapsed();
                match max_speed.checked_sub(time_since){
                    Some(time_dur) => time_dur,
                    None => Duration::from_secs(0),
                }
            },
            _ => Duration::from_secs(0),
        }
    }
}

pub enum AssetUpdateState{
    ///When the asset manager does nothing, aka. after initialisation.
    Idle,

    Working(Instant),
    ///The asset manager is waiting since some time
    Waiting(Instant),
}

impl AssetUpdateState{
    pub fn working() -> Self {
        AssetUpdateState::Working(Instant::now())
    }

    pub fn wait() -> Self{
        AssetUpdateState::Waiting(Instant::now())
    }

    ///Returns true if `max_speed` is smaller then the time since we wait
    pub fn should_update(&self, max_speed: Duration) -> bool{
        match self{
            &AssetUpdateState::Working(_) => false,
            &AssetUpdateState::Waiting(time) => {
                if time.elapsed() > max_speed{
                    true
                }else{
                    false
                }
            }
            &AssetUpdateState::Idle => true,
        }
    }

    ///Returns the duration left till we can update next, is Duration(0) if we can update
    pub fn duration_left(&self, max_speed: Duration) -> Duration{
        match self{
            &AssetUpdateState::Waiting(time) => {
                //time since time
                let time_since = time.elapsed();
                match max_speed.checked_sub(time_since){
                    Some(time_dur) => time_dur,
                    None => Duration::from_secs(0),
                }
            },
            _ => Duration::from_secs(0),
        }
    }
}

enum LastStep{
    Asset,
    Render,
    Physics
}

///Collects the duration which are needed for the systems to pass till a next iteration can
/// be started.
struct DurationCollection {
    pub render_duration: Duration,
    pub asset_duration: Duration,
    //physics_duration: Duration
}

///Keeps track of the current state of the sub models. Can be asked "what to do next" which will
/// return the logical next step in the rendering loop
pub struct EngineStateMachine{
    render_state: Arc<Mutex<RenderState>>,
    asset_state: Arc<Mutex<AssetUpdateState>>,
    engine_settings: Arc<Mutex<EngineSettings>>,
    duration_collection: DurationCollection,
    last_step: LastStep,
}

impl EngineStateMachine{
    pub fn new(
        render_state: Arc<Mutex<RenderState>>,
        asset_state: Arc<Mutex<AssetUpdateState>>,
        engine_settings: Arc<Mutex<EngineSettings>>,
    ) -> Self{

        let duration_collection = {
            let engine_lck = engine_settings.lock().expect("Failed to lock engine settings");
            DurationCollection{
                render_duration: Duration::from_secs(1).checked_div(
                    engine_lck.max_fps as u32
                ).expect("Failed to create engine duration"),
                asset_duration: Duration::from_secs(1).checked_div(
                    engine_lck.max_asset_updates as u32
                ).expect("Failed to create asset duration"),
            }
        };

        println!("DURATIONS=================================", );
        println!("{:?}", duration_collection.render_duration );
        println!("{:?}", duration_collection.asset_duration );
        println!("DURATIONS=================================", );

        EngineStateMachine{
            render_state,
            asset_state,
            engine_settings,
            duration_collection,
            last_step: LastStep::Asset,
        }
    }

    pub fn update(&mut self) -> NextStep{
        //Priorise the renderer
        let render_should_update = {
            let render_state_lck = self.render_state.lock().expect("failed to lock render_state");
            if render_state_lck.should_update(self.duration_collection.render_duration){
                //Only update if the asset manager or the physics are not currently working
                let asset_is_working = {
                    let asset_state_lck = self.asset_state.lock().expect("failed to lock asset state");
                    match *asset_state_lck{
                        AssetUpdateState::Working(_) => true,
                        _ => false,
                    }
                };

                if !asset_is_working{
                    //println!("Should update Renderer", );
                    true
                }else{
                    //println!("Should render, but asset manager is working", );
                    false
                }
            }else{
                //println!("Render not ready", );
                false
            }
        };
        //TODO add physics system, for now asset system
        let asset_should_update = {
            let asset_state_lck = self.asset_state.lock().expect("failed to lock asset_state");
            if asset_state_lck.should_update(self.duration_collection.asset_duration){
                //Only update if the renderer is currently occupyed on the gpu
                let render_working = {
                    let render_state_lck = self.render_state.lock().expect("failed to lock render state");
                    match *render_state_lck{
                        RenderState::IsWorkingOnGpu(_) => true,
                        _ => false,
                    }
                };
                if render_working{
                    //println!("Should update Assets", );
                    true
                }else{
                    //println!("Should update assets, but renderer is working", );
                    false
                }
            }else{
                //println!("Asset manager not ready", );
                false
            }
        };

        //Now decide. Try to let the asset manager follow the renderer, follow the asset manager etc.
        match self.last_step{
            LastStep::Asset => {
                if render_should_update{
                    self.last_step = LastStep::Render;
                    return NextStep::Render;
                }
            },
            LastStep::Render => {
                if asset_should_update{
                    self.last_step = LastStep::Asset;
                    return NextStep::UpdateAssets;
                }
            }
            LastStep::Physics => {
                //TODO we currently have no physics :)
                if asset_should_update{
                    self.last_step = LastStep::Asset;
                    return NextStep::UpdateAssets;
                }
            }
        }

        //Compare the durations left
        let asset_remain = {
            let asset_state_lck = self.asset_state.lock().expect("failed to lock asset state");
            asset_state_lck.duration_left(self.duration_collection.asset_duration)
        };

        let render_remain = {
            let render_state_lck = self.render_state.lock().expect("failed to lock render state");
            render_state_lck.duration_left(self.duration_collection.render_duration)
        };

        let mut remaining = Duration::from_secs(0);

        if asset_remain <= Duration::from_millis(0) {
            //println!("Asset time smaller", );
            remaining = render_remain;
        }else if render_remain <= Duration::from_millis(0){
            //println!("Render time smaller", );
            remaining = asset_remain;
        }else{
            //println!("Both bigger zero", );
            remaining = asset_remain.min(render_remain)
        }

        //println!("RenderRem: {:?} Assset_rem: {:?}", render_remain, asset_remain);
        //println!("Using: {:?}", remaining);
        //if both shouldn't be updated
        NextStep::Nothing(remaining)
    }

    pub fn asset_working(&mut self){
        let mut state = self.asset_state.lock().expect("failed to lock asset_state");
        *state = AssetUpdateState::working();
    }

    pub fn render_on_cpu(&mut self){
        let mut state = self.render_state.lock().expect("failed to lock asset_state");
        *state = RenderState::work_cpu();
    }

}
