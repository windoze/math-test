#![windows_subsystem = "windows"]

use log::info;
use slint::Weak;

slint::include_modules!();

const API_URL: &str = "http://m1u.0d0a.com:3001/api";

#[derive(Default, serde::Deserialize)]
struct QuestionResponse {
    id: i64,
    question: String,
    answer: Option<i64>,
}

async fn get_new_question(ui: Weak<AppWindow>) -> anyhow::Result<()> {
    let ui_clone = ui.clone();
    ui_clone.upgrade_in_event_loop(|ui| {
        ui.set_loading_overlay_visible(true);
        ui.set_question("".into());
        ui.set_answer("".into());
    })?;
    let client = reqwest::Client::new();
    let question: QuestionResponse = client
        .post(format!("{API_URL}/new-question"))
        .send()
        .await?
        .json()
        .await?;
    ui.upgrade_in_event_loop(move |ui| {
        info!(
            "Got new question, id: {}, question: {}, answer: {:?}",
            question.id, question.question, question.answer
        );
        ui.set_id(question.id.to_string().into());
        ui.set_question(format!("{} =", question.question).into());
        ui.set_answer(
            question
                .answer
                .map(|n| n.to_string())
                .unwrap_or_default()
                .into(),
        );
        ui.set_number_enabled(ui.get_answer().len() <= 8);
        ui.set_loading_overlay_visible(false);
    })?;
    Ok(())
}

async fn submit_answer(ui: Weak<AppWindow>, id: i64, answer: i64) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct SubmitAnswerRequest {
        id: i64,
        answer: i64,
    }

    #[derive(serde::Deserialize)]
    struct SubmitAnswerResponse {
        id: i64,
        correct: bool,
    }

    let request = SubmitAnswerRequest { id, answer };

    let client = reqwest::Client::new();
    let response: SubmitAnswerResponse = client
        .post(format!("{API_URL}/submit-answer"))
        .json(&request)
        .send()
        .await?
        .json()
        .await?;
    info!("Id: {}, correct: {}", response.id, response.correct);
    let ui_clone = ui.clone();
    ui_clone.upgrade_in_event_loop(move |ui| {
        if response.correct {
            ui.set_correct_overlay_visible(true);
        } else {
            ui.set_incorrect_overlay_visible(true);
        }
    })?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    get_new_question(ui).await?;
    ui_clone.upgrade_in_event_loop(move |ui| {
        ui.set_correct_overlay_visible(false);
        ui.set_incorrect_overlay_visible(false);
    })?;
    Ok(())
}

struct ConsoleHolder;

impl ConsoleHolder {
    /// Attach console from parent process.
    pub fn new() -> Self {
        #[cfg(windows)]
        unsafe {
            winapi::um::wincon::AttachConsole(0xFFFFFFFF);
        }
        Self
    }

    pub fn wrap<T>(self, t: T) -> T {
        t
    }
}

fn main() -> anyhow::Result<()> {
    let c = ConsoleHolder::new();

    env_logger::builder().init();
    let rt = tokio::runtime::Runtime::new()?;
    let handle = rt.handle().clone();

    let ui = AppWindow::new()?;
    handle.spawn(get_new_question(ui.as_weak()));

    let weak_ui = ui.as_weak();
    ui.on_num_clicked(move |num| {
        let ui = weak_ui.unwrap();
        let mut answer = ui.get_answer().parse::<i64>().unwrap_or(0);
        answer = answer * 10 + (num as i64);
        ui.set_answer(answer.to_string().into());
        ui.set_number_enabled(ui.get_answer().len() <= 8);
    });

    let weak_ui = ui.as_weak();
    ui.on_backspace_clicked(move || {
        let ui = weak_ui.unwrap();
        let mut answer = ui.get_answer().parse::<i64>().unwrap_or(0);
        if answer < 10 {
            ui.set_answer("".into());
        } else {
            answer /= 10;
            ui.set_answer(answer.to_string().into());
        }
        ui.set_number_enabled(ui.get_answer().len() <= 8);
    });

    let weak_ui = ui.as_weak();
    ui.on_submit_clicked(move || {
        let id = weak_ui.unwrap().get_id().parse::<i64>().unwrap_or(0);
        let answer = weak_ui.unwrap().get_answer().parse::<i64>().unwrap_or(0);
        handle.spawn(submit_answer(weak_ui.clone(), id, answer));
    });

    ui.run()?;

    c.wrap(());
    Ok(())
}
