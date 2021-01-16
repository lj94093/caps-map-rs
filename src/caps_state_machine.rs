use evdev_rs::{UInputDevice, ReadFlag, InputEvent};
use evdev_rs::enums::EventCode;
use std::collections::{HashSet, HashMap};
use evdev_rs::enums::EV_KEY::{KEY_U, KEY_I, KEY_O, KEY_J, KEY_K, KEY_L, KEY_N, KEY_M, KEY_HOME, KEY_UP, KEY_END, KEY_LEFT, KEY_DOWN, KEY_RIGHT, KEY_BACKSPACE, KEY_DELETE, KEY_CAPSLOCK};

enum CapsState{
    down,
    up,
    repeat
}
pub struct CapsStatMachine{
    caps_down:bool,
    shift_down:bool,
    accept_keys: HashSet<EventCode>,
    map_key:HashMap<(bool,EventCode),EventCode>
}

/*
             u_down|u_repeat
caps_down |      home   |




 */
impl CapsStatMachine{
    fn new()->CapsStatMachine{
        let accept_keys=[KEY_U,KEY_I,KEY_O,KEY_J,KEY_K,KEY_L,KEY_N,KEY_M].iter().clone().collect();
        let map_key:HashMap<(bool,EventCode),EventCode>=[
            ((true,KEY_U.into()),KEY_HOME.into()),
            ((true,KEY_I.into()),KEY_UP.into()),
            ((true,KEY_O.into()),KEY_END.into()),
            ((true,KEY_J.into()),KEY_LEFT.into()),
            ((true,KEY_K.into()),KEY_DOWN.into()),
            ((true,KEY_L.into()),KEY_RIGHT.into()),
            ((true,KEY_N.into()),KEY_BACKSPACE.into()),
            ((true,KEY_M.into()),KEY_DELETE.into()),
        ].iter().clone().collect();
        CapsStatMachine{
            caps_down:false,
            shift_down:false,
            accept_keys,
            map_key
        }
    }

    fn transform(mut self,input:InputEvent) -> Option<InputEvent>{
        if input.event_code==KEY_CAPSLOCK {
            self.caps_down=!self.caps_down;
            return input;
        }

        let output=match self.map_key.get(&(self.caps_down,input.event_code.clone())){
            Some(output) => {
                let mut output_event=input.clone();
                output_event.event_code=output.clone();
                output_event
            },
            None => input.clone()
        };
        Some(output)
    }
}