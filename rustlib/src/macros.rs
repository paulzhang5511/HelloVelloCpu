#[macro_export]
macro_rules! rustlib_main {
    ($app_type:ty, $java_class:expr) => {
        #[unsafe(no_mangle)]
        #[cfg(target_os = "android")]
        pub extern "system" fn JNI_OnLoad(
            vm: ::jni::JavaVM,
            _reserved: *mut ::std::ffi::c_void,
        ) -> ::jni::sys::jint {
            let mut env = vm.get_env().expect("无法获取 JNIEnv");
            crate::internal::register_natives::<$app_type>(&mut env, $java_class);
            ::jni::JNIVersion::V6.into()
        }
    };
}
