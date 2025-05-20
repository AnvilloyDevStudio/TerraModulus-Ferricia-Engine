/*
 * SPDX-FileCopyrightText: 2025 TerraModulus Team and Contributors
 * SPDX-License-Identifier: LGPL-3.0-only
 */

fn main() {
	if cfg!(target_os = "linux") {
		println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
	} else if cfg!(target_os = "macos") {
		println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
	}
}
