use std::env;
use std::fs;
use std::path::Path;

static SHADER_SRC_DIR: &str = "shaders/";
static SHADER_OUT_DIR_NAME: &str = "assets/shaders";

fn main() {
    println!("cargo::rerun-if-changed={}", SHADER_SRC_DIR);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let shader_src = fs::read_dir(SHADER_SRC_DIR).unwrap();
    let shader_out_dir = Path::new(&manifest_dir).join(SHADER_OUT_DIR_NAME);

    if shader_out_dir.exists() {
        fs::remove_dir_all(&shader_out_dir).unwrap();
    }
    fs::create_dir_all(&shader_out_dir).unwrap();

    println!("cargo::rustc-env=SHADER_OUT_DIR={}", SHADER_OUT_DIR_NAME);

    let compiler = shaderc::Compiler::new().unwrap();

    for entry in shader_src {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().unwrap();
        let shader_kind = match ext.to_str().unwrap() {
            "vert" => shaderc::ShaderKind::Vertex,
            "frag" => shaderc::ShaderKind::Fragment,
            _ => continue,
        };

        let file_name = path.file_name().unwrap();
        let binary_shader_path = shader_out_dir.join(file_name).with_added_extension("spv");
        let artifact_result = compiler.compile_into_spirv(
            &fs::read_to_string(&path).unwrap(),
            shader_kind,
            file_name.to_str().unwrap(),
            "main",
            None,
        );
        let artifact = match artifact_result {
            Ok(a) => a,
            Err(e) => {
                // 发生错误时，明确打印出是哪个文件报错，以及具体的源码和错误信息
                eprintln!("=======================================================");
                eprintln!("🔥 SHADER COMPILATION FAILED!");
                eprintln!("📄 File: {}", path.display());
                eprintln!("❌ Error: {:?}", e);
                eprintln!("=======================================================");
                panic!("Failed to compile shader: {:?}", binary_shader_path);
            }
        };
        fs::write(binary_shader_path, artifact.as_binary_u8()).unwrap();
    }
}
