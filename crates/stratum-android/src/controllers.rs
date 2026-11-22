//! Android InputDevice discovery, kept at the JNI boundary in the native host.
use horizon_client::controller_ui::Device;
use jni::{
    JavaVM,
    objects::{JIntArray, JString, JValue},
};
use std::sync::OnceLock;
static VM: OnceLock<JavaVM> = OnceLock::new();
pub fn install(
    app: &winit::platform::android::activity::AndroidApp,
) -> Result<(), jni::errors::Error> {
    // AndroidApp owns this live process VM throughout android_main. JavaVM does not destroy it.
    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast())? };
    let _ = VM.set(vm);
    horizon_client::android::install_controller_inventory(devices);
    Ok(())
}
fn devices() -> Vec<(i32, Device)> {
    let Some(vm) = VM.get() else {
        return Vec::new();
    };
    let Ok(mut env) = vm.attach_current_thread() else {
        return Vec::new();
    };
    let result: Result<_, jni::errors::Error> = env.with_local_frame(64, |env| {
        let ids = env
            .call_static_method("android/view/InputDevice", "getDeviceIds", "()[I", &[])?
            .l()?;
        let ids = JIntArray::from(ids);
        let mut values = vec![0; env.get_array_length(&ids)? as usize];
        env.get_int_array_region(&ids, 0, &mut values)?;
        let mut devices = Vec::new();
        for id in values {
            env.with_local_frame(16, |env| -> Result<(), jni::errors::Error> {
                let device = env
                    .call_static_method(
                        "android/view/InputDevice",
                        "getDevice",
                        "(I)Landroid/view/InputDevice;",
                        &[JValue::Int(id)],
                    )?
                    .l()?;
                if device.is_null() {
                    return Ok(());
                }
                let sources = env.call_method(&device, "getSources", "()I", &[])?.i()?;
                if sources & (0x400 | 0x10) == 0 {
                    return Ok(());
                }
                let name = JString::from(
                    env.call_method(&device, "getName", "()Ljava/lang/String;", &[])?
                        .l()?,
                );
                let name: String = env.get_string(&name)?.into();
                let vendor = env.call_method(&device, "getVendorId", "()I", &[])?.i()? as u16;
                let product = env.call_method(&device, "getProductId", "()I", &[])?.i()? as u16;
                devices.push((
                    id,
                    Device {
                        name,
                        vendor: Some(vendor),
                        product: Some(product),
                    },
                ));
                Ok(())
            })?;
        }
        Ok(devices)
    });
    if result.is_err() {
        let _ = env.exception_clear();
    }
    result.unwrap_or_default()
}
