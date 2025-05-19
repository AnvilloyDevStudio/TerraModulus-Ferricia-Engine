use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_terramodulus_engine_ferricia_Demo_hello(mut env: JNIEnv, class: JClass, name: JString) -> jstring {
	let input: String =
		env.get_string(&name).expect("Couldn't get java string!").into();
	let output = env.new_string(format!("Hello, {}!", input))
		.expect("Couldn't create java string!");
	output.into_raw()
}
