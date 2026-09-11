//! 빌드 스크립트: Windows 실행 파일에 앱 아이콘 리소스를 임베드한다.
//!
//! gpui(gpui-pre)의 Windows 플랫폼은 실행 파일에서
//! `LoadImageW(..., MAKEINTRESOURCE(1), IMAGE_ICON, ...)`로 창 아이콘을 읽으므로,
//! `assets/app.rc`가 `assets/icon.ico`를 리소스 ID 1로 등록한다.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    use embed_resource::CompilationResult;
    match embed_resource::compile("assets/app.rc", embed_resource::NONE) {
        CompilationResult::Ok => {}
        CompilationResult::Failed(err) => panic!("아이콘 리소스 컴파일 실패: {err}"),
        result => println!("cargo:warning=아이콘 리소스를 임베드하지 못했습니다: {result}"),
    }
}
