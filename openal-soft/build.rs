/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */
use std::{env, fs};
use std::path::PathBuf;

fn main() {
	let dst = cmake::Config::new("openal-soft-src")
		.define("ODE_WITH_DEMOS", "OFF")
		.build();

	println!("cargo:rustc-link-search=native={}", dst.display());
	println!("cargo:rustc-link-lib=OpenAL32");

	let bindings = bindgen::Builder::default()
		.headers(fs::read_dir(dst.join("include/AL").to_str().unwrap()).unwrap()
			.map(|v| v.unwrap().path().to_string_lossy().into_owned()))
		.clang_arg(format!("-I{}", dst.join("include").to_str().unwrap()))
		.generate().unwrap();

	let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
	bindings.write_to_file(out_path.join("bindings.rs")).unwrap();
	println!("cargo:rerun-if-changed=openal-soft-src");
}
