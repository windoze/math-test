use log::info;
use once_cell::sync::OnceCell;
use slint::Weak;

slint::include_modules!();

static INSTANCE: OnceCell<quiz_repo::QuizRepo> = OnceCell::new();

async fn get_new_question(ui: Weak<AppWindow>) -> anyhow::Result<()> {
    let ui_clone = ui.clone();
    ui_clone.upgrade_in_event_loop(|ui| {
        ui.set_loading_overlay_visible(true);
        ui.set_question("".into());
        ui.set_answer("".into());
    })?;
    let question = INSTANCE
        .get()
        .ok_or(anyhow::anyhow!("Failed to get instance"))?
        .new_question()
        .await?;
    ui.upgrade_in_event_loop(move |ui| {
        info!(
            "Got new question, id: {}, question: {}, answer: {:?}",
            question.get_id(),
            question.get_question(),
            question.get_answer(),
        );
        ui.set_id(question.get_id().to_string().into());
        ui.set_question(format!("{} =", question.get_question()).into());
        ui.set_answer(
            question
                .get_answer()
                .map(|n| n.to_string())
                .unwrap_or_default()
                .into(),
        );
        ui.set_loading_overlay_visible(false);
    })?;
    Ok(())
}

async fn submit_answer(ui: Weak<AppWindow>, id: i64, answer: i64) -> anyhow::Result<()> {
    let correct = INSTANCE
        .get()
        .ok_or(anyhow::anyhow!("Failed to get instance"))?
        .answer_question(id, answer)
        .await?;
    info!("Id: {}, correct: {}", id, correct);
    let ui_clone = ui.clone();
    ui_clone.upgrade_in_event_loop(move |ui| {
        if correct {
            ui.set_correct_overlay_visible(true);
        } else {
            ui.set_incorrect_overlay_visible(true);
        }
    })?;
    tokio::join!(
        tokio::time::sleep(std::time::Duration::from_secs(1)),
        get_new_question(ui)
    )
    .1?;
    ui_clone.upgrade_in_event_loop(move |ui| {
        ui.set_correct_overlay_visible(false);
        ui.set_incorrect_overlay_visible(false);
    })?;
    Ok(())
}

fn update_score(ui: AppWindow, score: Vec<(String, i64, i64, f64)>) {
    let row_data: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for s in score {
        let items = Rc::new(VecModel::default());
        items.push(StandardListViewItem::from(SharedString::from(s.0)));
        items.push(StandardListViewItem::from(SharedString::from(
            s.1.to_string(),
        )));
        items.push(StandardListViewItem::from(SharedString::from(
            s.2.to_string(),
        )));
        items.push(StandardListViewItem::from(SharedString::from(
            s.3.to_string(),
        )));
        row_data.push(items.into());
    }
    ui.set_scores(row_data.into());
}

fn update_mistake_collection(ui: AppWindow, mistake_collection: Vec<(i64, String, Option<i64>)>) {
    let row_data: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for s in mistake_collection {
        let items = Rc::new(VecModel::default());
        items.push(StandardListViewItem::from(SharedString::from(s.1)));
        items.push(StandardListViewItem::from(SharedString::from(
            s.2.map(|n| n.to_string()).unwrap_or_default(),
        )));
        row_data.push(items.into());
    }
    ui.set_mistakes(row_data.into());
}

async fn overlay_wrapper<T: Send + Sized + 'static>(
    ui: Weak<AppWindow>,
    fut: impl std::future::Future<Output = anyhow::Result<T>>,
    updater: impl FnOnce(AppWindow, T) + Send + 'static,
) -> anyhow::Result<()> {
    // Show loading overlay
    let ui_clone = ui.clone();
    ui_clone.upgrade_in_event_loop(|ui| {
        ui.set_loading_overlay_visible(true);
    })?;
    // Run the future
    let result = fut.await?;
    // Update UI and hide loading overlay
    ui.upgrade_in_event_loop(move |ui| {
        updater(ui.clone_strong(), result);
        ui.set_loading_overlay_visible(false);
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

pub fn ui_main<P>(p: Option<P>) -> anyhow::Result<()>
where
    P: AsRef<std::path::Path>,
{
    let c = ConsoleHolder::new();

    let rt = tokio::runtime::Runtime::new()?;
    let handle = rt.handle().clone();

    let db_path = p.or_else(|| dirs::config_dir().map(|d| d.join("math-quiz.db")));

    let instance = handle.block_on(quiz_repo::QuizRepo::new(db_path))?;

    INSTANCE.set(instance).ok();

    let ui = AppWindow::new()?;
    handle.spawn(get_new_question(ui.as_weak()));

    let weak_ui = ui.as_weak();
    let handle_clone = handle.clone();
    ui.on_submit_clicked(move || {
        let id = weak_ui.unwrap().get_id().parse::<i64>().unwrap_or(0);
        let answer = weak_ui.unwrap().get_answer().parse::<i64>().unwrap_or(0);
        handle_clone.spawn(submit_answer(weak_ui.clone(), id, answer));
    });

    let weak_ui = ui.as_weak();
    ui.on_tab_changed(move |n| {
        debug!("Tab changed: {}", n);
        if n == 1 {
            handle.spawn(overlay_wrapper(
                weak_ui.clone(),
                INSTANCE
                    .get()
                    .expect("Failed to get instance")
                    .get_all_localtime_daily_statistics(),
                update_score,
            ));
        } else if n == 2 {
            handle.spawn(overlay_wrapper(
                weak_ui.clone(),
                INSTANCE
                    .get()
                    .expect("Failed to get instance")
                    .mistake_collection(),
                update_mistake_collection,
            ));
        }
    });

    ui.run()?;

    c.wrap(Ok(()))
}
