use rofi::RofiPlugin;

fn params_test(rofi: &RofiPlugin, params: Vec<String>) {
    println!("test, params: {:?}", params)
}

fn no_params_test(rofi: &RofiPlugin, params: Vec<String>) {
    rofi.add_menu_entry("l2", no_params_test_l2);
    rofi.add_menu_entry_with_params(
        "l2_pre_params",
        no_pre_params_test_l2,
        vec![String::from("asdasd")],
    );
}

fn no_params_test_l2(rofi: &RofiPlugin, params: Vec<String>) {
    println!("no_params_l2")
}

fn no_pre_params_test_l2(rofi: &RofiPlugin, params: Vec<String>) {
    println!("no_pre_params_l2, params: {:?}", params)
}

fn entrypoint(rofi: &RofiPlugin, params: Vec<String>) {
    rofi.add_menu_entry("params", params_test);
    rofi.add_menu_entry("no-params", no_params_test);
}

fn main() {
    let mut rofi = RofiPlugin::new();
    rofi.register_entrypoint(entrypoint);
    rofi.register_callback_with_params(params_test, vec![String::from("asdasd")]);
    rofi.register_callback(no_params_test);
    rofi.register_callback(no_params_test_l2);
    rofi.register_callback_with_params(no_pre_params_test_l2, vec![String::from("")]);
    rofi.run();
}
