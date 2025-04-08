use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "GotchaWidgetLib"])]
    async fn setup();

    #[wasm_bindgen(js_namespace = ["window", "GotchaWidgetLib"])]
    async fn onChallengeResponse(success: bool);

    #[wasm_bindgen(js_namespace = ["window", "GotchaWidgetLib"])]
    async fn onChallengeError();
}

#[wasm_bindgen]
pub async fn init() {
    setup().await;
}

#[wasm_bindgen]
pub async fn send_challenge_result(success: bool) {
    onChallengeResponse(success).await;
}

#[wasm_bindgen]
pub async fn send_challenge_error() {
    onChallengeError().await;
}
