mod caps_state_machine;
extern crate udev;
extern crate evdev_rs;
extern crate libc;
use evdev_rs::enums::EV_KEY::{KEY_CAPSLOCK, KEY_LEFTSHIFT, KEY_RIGHTSHIFT, KEY_U, KEY_I, KEY_O, KEY_J, KEY_K, KEY_L, KEY_N, KEY_M};
use evdev_rs::GrabMode::Grab;
use evdev_rs::{UInputDevice, ReadFlag, ReadStatus};
use std::path::Path;
use evdev_rs::enums::{EventType, EventCode};
use std::thread::sleep;
use std::time::Duration;
use core::{mem, ptr};
use std::os::unix::io::AsRawFd;

fn event_loop(keyboard_path :Box<Path>){
    let keyboard_file =match std::fs::File::open(&keyboard_path){
        Err(_) => {
            println!("Failed to open {:?}",keyboard_path);
            return;
        },
        Ok(f) => f
    };

    let mut device= match evdev_rs::Device::new_from_fd(keyboard_file){
        Ok(d) => d,
        Err(err) => {
            println!("Failed to create evdev device from path:{:?} err:{}",keyboard_path,err);
            return;
        }
    };
    device.grab(Grab).expect("grab the device failed");
    device.enable(&EventType::EV_KEY).expect("enable the EV_KEY failed");
    let accept_keys=[KEY_CAPSLOCK,KEY_LEFTSHIFT,KEY_RIGHTSHIFT,KEY_U,KEY_I,KEY_O,KEY_J,KEY_K,KEY_L,KEY_N,KEY_M];
    for key in accept_keys.iter(){
        device.enable(&EventCode::EV_KEY(key.clone())).expect(format!("enable the key:{:?} failed",key).as_str());
    }
    let uinput_device=UInputDevice::create_from_device(&device).expect("create UInputDevice failed");
    let mut csm=caps_state_machine::CapsStateMachine::new();
    loop{
        let (status,input_event)=match device.next_event(ReadFlag::NORMAL|ReadFlag::BLOCKING){
            Ok((s,i)) => (s,i),
            Err(_) => continue
        };
        if status==ReadStatus::Sync{
            continue;
        }
        if let Some(output_event)=csm.transform(input_event){
            uinput_device.write_event(&output_event).expect("write event error");
        }

        sleep(Duration::from_millis(5));
    }
}

fn get_keyboard_path(device: udev::Device) -> Option<Box<Path>> {
    if device.syspath().starts_with("/sys/devices/virtual/input/") {
        return None;
    }
    let devnode_path=match device.devnode(){
        Some(devnode_path) if devnode_path.to_str().unwrap().starts_with("/dev/input/event") => devnode_path,
        _ => return None
    };

    let devnode_file=match std::fs::File::open(&devnode_path){
        Err(_e) => {
            println!("Failed to open {:?}",devnode_path);
            return None;
        },
        Ok(f) => f
    };

    let device_evdev= match evdev_rs::Device::new_from_fd(devnode_file){
        Ok(d) => d,
        Err(err) => {
            println!("Failed to create evdev device from path:{:?} err:{}",devnode_path,err);
            return None;
        }
    };
    if device_evdev.has(&EventType::EV_KEY) && device_evdev.has(&EventCode::EV_KEY(KEY_CAPSLOCK)) {
        println!("get a keyboard path:{:?}",devnode_path);
        return Some(Box::from(devnode_path));
    }
    return None;
}
unsafe fn has_event(raw_fd: i32) -> bool{
    let ref mut fds={
        let mut raw_fd_set = mem::MaybeUninit::<libc::fd_set>::uninit().assume_init();
        libc::FD_ZERO(&mut raw_fd_set);
        raw_fd_set
    };
    libc::FD_SET(raw_fd,fds);
    return libc::select(raw_fd+1, fds,ptr::null_mut(),ptr::null_mut(),ptr::null_mut())>0
        && libc::FD_ISSET(raw_fd,fds as *mut libc::fd_set);
}

fn monitor_keyboard_add(){
    // monitor the device add/remove,if it's a keyboard,we also need to deal it.
    let mut event_socket=udev::MonitorBuilder::new().expect("can't create a monitor")
        .match_subsystem("input").expect("match subsystem input failed")
        .listen().expect("create socket from monitor error");
    // device action loop
    loop{
        let raw_fd=event_socket.as_raw_fd();
        if unsafe{has_event(raw_fd)} {
            let event = match event_socket.next() {
                Some(e) if e.event_type()==udev::EventType::Add=> e,
                _ => continue,
            };
            if let Some(keyboard_path)= get_keyboard_path(event.device()){
                // create a thread to receive keyboard event, don't block the device action listening.
                println!("add a thread to monitor keyboard event");
                std::thread::spawn(move ||{
                    event_loop(keyboard_path);
                });
            }
        }
    }
}

fn main() {
    let mut enumerator = udev::Enumerator::new().unwrap();

    enumerator.match_subsystem("input").unwrap();
    // get current keyboard device.
    for device in enumerator.scan_devices().unwrap() {
        // println!("found device: {:?}", device.syspath());
        // There may be more than one keyboard connected,so we need to spawn a thread for every keyboard.
        if let Some(keyboard_path)= get_keyboard_path(device) {
            std::thread::spawn(||{
                event_loop(keyboard_path);
            });
        };
    }
    monitor_keyboard_add();
}