fn main() {
    embed_resource::compile("math-quiz.rc", embed_resource::NONE);
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles);
    slint_build::compile_with_config("ui/appwindow.slint", config).unwrap();
}
