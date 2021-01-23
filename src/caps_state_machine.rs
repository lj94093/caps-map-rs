use evdev_rs::{InputEvent};
use evdev_rs::enums::{EventCode, EV_KEY, int_to_ev_key};
use std::collections::{HashMap};
use evdev_rs::enums::EV_KEY::{KEY_U, KEY_I, KEY_O, KEY_J, KEY_K, KEY_L, KEY_N, KEY_M, KEY_HOME, KEY_UP, KEY_END, KEY_LEFT, KEY_DOWN, KEY_RIGHT, KEY_BACKSPACE, KEY_DELETE, KEY_CAPSLOCK};

pub struct CapsStateMachine {
    caps_down:bool,
    map_key:HashMap<(bool,EV_KEY),EV_KEY>
}

impl CapsStateMachine {
    pub fn new()-> CapsStateMachine {
        let mut map_key:HashMap<(bool,EV_KEY),EV_KEY>=HashMap::new();
        map_key.insert((true,KEY_U),KEY_HOME);
        map_key.insert((true,KEY_I),KEY_UP);
        map_key.insert((true,KEY_O),KEY_END);
        map_key.insert((true,KEY_J),KEY_LEFT);
        map_key.insert((true,KEY_K),KEY_DOWN);
        map_key.insert((true,KEY_L),KEY_RIGHT);
        map_key.insert((true,KEY_N),KEY_BACKSPACE);
        map_key.insert((true,KEY_M),KEY_DELETE);
        CapsStateMachine {
            caps_down:false,
            map_key
        }
    }

    pub fn transform(&mut self,input:InputEvent) -> Option<InputEvent>{
        if input.event_code==EventCode::EV_KEY(KEY_CAPSLOCK) {
            self.caps_down=match input.value{
                0 => false,//release key
                1 => true,//press key down
                2 => true,//repeat
                _ => false
            };
            return None;
        }
        let (_key_type,key_value)=evdev_rs::util::event_code_to_int(&input.event_code);
        let key_value=int_to_ev_key(key_value).unwrap();
        match self.map_key.get(&(self.caps_down,key_value)){
            Some(output_key) => {
                let mut output_event=input.clone();
                output_event.event_code=EventCode::EV_KEY(output_key.clone());
                Some(output_event)
            }
            None => Some(input.clone())
        }
    }
}