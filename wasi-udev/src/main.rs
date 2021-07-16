mod models;

use models::wrappers::{
    udev_device::{
        get_attribute_value, get_devnode, get_devpath, get_driver, get_parent, get_property_value,
        get_subsystem, get_sysname, DeviceExt,
    },
    udev_enumerator::Enumerator,
};

fn main() {
    println!("Hello, world!");
}
