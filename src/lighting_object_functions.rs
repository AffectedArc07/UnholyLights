use byondapi::{
    parse_args,
    value::ByondValue, prelude::ByondValueList, typecheck_trait::ByondTypeCheck,
};
use num_traits::ToPrimitive;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use rayon::prelude::*;

#[derive(Clone)]
struct LightingObject {
    // Internal
    worked: bool,

    // Inputs
    ccr: f32,
    ccg: f32,
    ccb: f32,
    cca: f32,

    crcr: f32,
    crcb: f32,
    crcg: f32,

    cgcr: f32,
    cgcb: f32,
    cgcg: f32,

    cbcr: f32,
    cbcb: f32,
    cbcg: f32,

    cacr: f32,
    cacb: f32,
    cacg: f32,

    // Outputs
    is_luminous: bool,
    icon_state: Option<String>,
    colour_list: Option<Vec<f32>>
}

impl LightingObject {
    pub fn new() -> Self {
        Self {
            worked: false,
            ccr: 0.0f32,
            ccg: 0.0f32,
            ccb: 0.0f32,
            cca: 0.0f32,
            crcr: 0.0f32,
            crcb: 0.0f32,
            crcg: 0.0f32,
            cgcr: 0.0f32,
            cgcg: 0.0f32,
            cgcb: 0.0f32,
            cbcr: 0.0f32,
            cbcg: 0.0f32,
            cbcb: 0.0f32,
            cacr: 0.0f32,
            cacb: 0.0f32,
            cacg: 0.0f32,
            is_luminous: false,
            icon_state: None,
            colour_list: None
        }
    }

    pub fn from_byond(mut self, bval: ByondValue) -> Self {
        let my_turf = bval.read_var("myturf").unwrap();
        let corners_var = my_turf.read_var("corners").unwrap();

        let has_corners = corners_var.is_list();
        let mut cr: Option<ByondValue> = None;
        let mut cg: Option<ByondValue> = None;
        let mut cb: Option<ByondValue> = None;
        let mut ca: Option<ByondValue> = None;

        if has_corners {
            let corners: ByondValueList = my_turf.read_var("corners").unwrap().try_into().unwrap();
        
            if !corners[2].is_null() {
                cr = Some(corners[2].clone());
            }
            
            if !corners[1].is_null() {
                cg = Some(corners[1].clone());
            }
        
            if !corners[3].is_null() {
                cb = Some(corners[3].clone());
            }
        
            if !corners[0].is_null() {
                ca = Some(corners[0].clone());
            }
        }

        if cr.is_some() {
            let cro = cr.unwrap();
            self.ccr = cro.read_var("cache_mx").unwrap().get_number().unwrap();
            self.crcr = cro.read_var("cache_r").unwrap().get_number().unwrap();
            self.crcg = cro.read_var("cache_g").unwrap().get_number().unwrap();
            self.crcb = cro.read_var("cache_b").unwrap().get_number().unwrap();
        }

        if cg.is_some() {
            let cgo = cg.unwrap();
            self.ccg = cgo.read_var("cache_mx").unwrap().get_number().unwrap();
            self.cgcr = cgo.read_var("cache_r").unwrap().get_number().unwrap();
            self.cgcg = cgo.read_var("cache_g").unwrap().get_number().unwrap();
            self.cgcb = cgo.read_var("cache_b").unwrap().get_number().unwrap();
        }

        if cb.is_some() {
            let cbo = cb.unwrap();
            self.ccb = cbo.read_var("cache_mx").unwrap().get_number().unwrap();
            self.cbcr = cbo.read_var("cache_r").unwrap().get_number().unwrap();
            self.cbcg = cbo.read_var("cache_g").unwrap().get_number().unwrap();
            self.cbcb = cbo.read_var("cache_b").unwrap().get_number().unwrap();
        }

        if ca.is_some() {
            let cao = ca.unwrap();
            self.cca = cao.read_var("cache_mx").unwrap().get_number().unwrap();
            self.cacr = cao.read_var("cache_r").unwrap().get_number().unwrap();
            self.cacg = cao.read_var("cache_g").unwrap().get_number().unwrap();
            self.cacb = cao.read_var("cache_b").unwrap().get_number().unwrap();
        }

        self
    }

    pub fn do_work(mut self) -> Self {
        // Handle the max
        let rg_max = self.ccr.max(self.ccg);
        let ba_max = self.ccb.max(self.cca);

        let lum_max = rg_max.max(ba_max);

        let is_luminous = lum_max > 0f32;

        if self.crcr + self.crcg + self.crcb + self.cgcr + self.cgcg + self.cgcb + self.cbcr + self.cbcg + self.cbcb + self.cacr + self.cacg + self.cacb == 12f32 {
            self.icon_state = Some("transparent".to_string());
        } else if !is_luminous {
            self.icon_state = Some("dark".to_string());
        } else {
            self.icon_state = None;
            self.colour_list = Some(vec![
                self.crcr, self.crcg, self.crcb, 0f32,
                self.cgcr, self.cgcg, self.cgcb, 0f32,
                self.cbcr, self.cbcg, self.cbcb, 0f32,
                self.cacr, self.cacg, self.cacb, 0f32,
                0f32, 0f32, 0f32, 1f32
            ]);
        }

        self.worked = true;

        self
    }

    pub fn write(self, val: &mut ByondValue) {
        match self.icon_state {
            Some(v) => val.write_var("icon_state", &ByondValue::new_str(v).unwrap()).unwrap(),
            None => val.write_var("icon_state", &ByondValue::new()).unwrap()
        }

        match self.colour_list {
            Some(v) => {
                let mut byond_colour_list: ByondValueList = ByondValue::new_list().unwrap().try_into().unwrap();
                for entry in v.into_iter() {
                    byond_colour_list.push(&ByondValue::new_num(entry));
                }
            }
            None => val.write_var("color", &ByondValue::new()).unwrap()
        }

        match self.is_luminous {
            true => val.write_var("luminosity", &ByondValue::new_num(1f32)).unwrap(),
            false => val.write_var("luminosity", &ByondValue::new_num(0f32)).unwrap()
        }

        val.write_var("needs_update", &ByondValue::new_num(0f32)).unwrap();
    }
}

thread_local! {
    static RUST_OBJECT_QUEUE: RefCell<HashMap<u32, LightingObject>> = RefCell::new(HashMap::new());
    static BYOND_OBJECT_QUEUE: RefCell<HashMap<u32, ByondValue>> = RefCell::new(HashMap::new());
}

#[no_mangle]
pub unsafe extern "C" fn qsize_obj(
    _argc: byondapi_sys::u4c,
    _argv: *mut ByondValue,
) -> ByondValue {

    let mut qsize = 0f32;

    BYOND_OBJECT_QUEUE.with(|cell| -> _ {
        qsize = match cell.borrow().len().to_f32() {
            Some(v) => v,
            None => 0f32
        }
    });

    return ByondValue::new_num(qsize);
}

#[no_mangle]
pub unsafe extern "C" fn dowork_obj(
    _argc: byondapi_sys::u4c,
    _argv: *mut ByondValue,
) -> ByondValue {

    //let mut processed_refs: Vec<u32> = Vec::new();
    let processed_lock = Arc::new(Mutex::new(Vec::new()));

    // First do the work
    RUST_OBJECT_QUEUE.with(|cell| -> _ {
        cell.borrow_mut().par_iter_mut().for_each(|o| -> _ {
            let worked_obj = o.1.do_work();
            let mut lock = processed_lock.lock().unwrap();
            lock.push(worked_obj);
            drop(lock);
        });
    });

    return ByondValue::new();
}


#[no_mangle]
pub unsafe extern "C" fn writelock_obj(
    argc: byondapi_sys::u4c,
    argv: *mut ByondValue,
) -> ByondValue {

    // First do the work
    RUST_OBJECT_QUEUE.with(|rust_cell| -> _ {
        BYOND_OBJECT_QUEUE.with(|byond_cell| -> _ {
            let mut processed_objects: Vec<u32> = Vec::new();

            for rust_object in rust_cell.borrow_mut().iter() {
                // We only care if we are worked
                if rust_object.1.worked {
                    // Grab the ref
                    let object_ref = rust_object.0;
                    // Grab the BYOND object
                    let byond_object = byond_cell.borrow().get(object_ref).unwrap();
                    // Write from rust to BYOND object
                    //rust_object.1.write(byond_object);
                    // Mark us as processed
                    processed_objects.push(object_ref.to_owned());
                }
            }

            // Now flush out
            for byond_ref in processed_objects {
                rust_cell.borrow_mut().remove(&byond_ref);
                byond_cell.borrow_mut().remove(&byond_ref);
            }
        });
    });

    return ByondValue::new();
}

#[no_mangle]
pub unsafe extern "C" fn queue_object(
    argc: byondapi_sys::u4c,
    argv: *mut ByondValue,
) -> ByondValue {
    let args = parse_args(argc, argv);
    let the_obj = args[0].clone();

    let object_ref = the_obj.get_ref().unwrap();

    
    RUST_OBJECT_QUEUE.with(|cell| -> _ {
        let lo: LightingObject = LightingObject::new();
        cell.borrow_mut().insert(object_ref, lo.from_byond(the_obj.clone()));
    });

    BYOND_OBJECT_QUEUE.with(|cell| -> _ {
        cell.borrow_mut().insert(object_ref, the_obj);
    });


    return ByondValue::new();
}

#[no_mangle]
pub unsafe extern "C" fn dequeue_object(
    argc: byondapi_sys::u4c,
    argv: *mut ByondValue,
) -> ByondValue {
    let args = parse_args(argc, argv);
    let the_obj = args[0].clone();

    BYOND_OBJECT_QUEUE.with(|cell| -> _ {
        cell.borrow_mut().remove(&the_obj.get_ref().unwrap());
    });


    return ByondValue::new();
}