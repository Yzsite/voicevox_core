use std::sync::Arc;

use crate::common::{throw_if_err, RUNTIME};
use jni::{
    objects::{JObject, JString},
    sys::jobject,
    JNIEnv,
};

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_VoiceModel_rsFromPath<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
    model_path: JString<'local>,
) {
    throw_if_err(env, (), |env| {
        let model_path = env.get_string(&model_path)?;
        let model_path = model_path.to_str()?;

        let internal = RUNTIME.block_on(voicevox_core::VoiceModel::from_path(model_path))?;

        unsafe { env.set_rust_field(&this, "internal", Arc::new(internal)) }?;

        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_VoiceModel_rsGetId<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<voicevox_core::VoiceModel>>(&this, "internal")?
                .clone()
        };

        let id = internal.id().raw_voice_model_id();

        let id = env.new_string(id)?;

        Ok(id.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_VoiceModel_rsGetMetasJson<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) -> jobject {
    throw_if_err(env, std::ptr::null_mut(), |env| {
        let internal = unsafe {
            env.get_rust_field::<_, _, Arc<voicevox_core::VoiceModel>>(&this, "internal")?
                .clone()
        };

        let metas = internal.metas();
        let metas_json = serde_json::to_string(&metas)?;
        Ok(env.new_string(metas_json)?.into_raw())
    })
}

#[no_mangle]
pub extern "system" fn Java_jp_Hiroshiba_VoicevoxCore_VoiceModel_rsDrop<'local>(
    env: JNIEnv<'local>,
    this: JObject<'local>,
) {
    throw_if_err(env, (), |env| {
        let internal =
            unsafe { env.get_rust_field::<_, _, voicevox_core::VoiceModel>(&this, "internal") }?;
        drop(internal);
        unsafe { env.take_rust_field(&this, "internal") }?;
        Ok(())
    })
}