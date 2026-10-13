use aside_lib::error::AppError;
use aside_lib::store::{Store, subjects};
use tempfile::TempDir;

fn open() -> (TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("data.db")).unwrap();
    (dir, store)
}

fn reopen(dir: &TempDir) -> Store {
    Store::open(&dir.path().join("data.db")).unwrap()
}

fn names(store: &Store, include_archived: bool) -> Vec<String> {
    store
        .read(|c| subjects::list(c, include_archived))
        .unwrap()
        .into_iter()
        .map(|s| s.name)
        .collect()
}

fn id_of(store: &Store, name: &str) -> i64 {
    store
        .read(|c| subjects::list(c, true))
        .unwrap()
        .into_iter()
        .find(|s| s.name == name)
        .unwrap()
        .id
}

fn assert_invalid<T: std::fmt::Debug>(result: Result<T, AppError>, needle: &str) {
    match result {
        Err(AppError::Invalid(msg)) => assert!(msg.contains(needle), "提示“{msg}”里应该包含“{needle}”"),
        other => panic!("应该被拒绝，实际是 {other:?}"),
    }
}

#[test]
fn fresh_database_has_five_presets_and_only_other_is_fallback() {
    let (_dir, store) = open();
    let list = store.read(|c| subjects::list(c, false)).unwrap();
    let summary: Vec<(&str, &str, bool)> =
        list.iter().map(|s| (s.name.as_str(), s.color.as_str(), s.is_fallback)).collect();
    assert_eq!(
        summary,
        vec![
            ("政治", "#C98B7B", false),
            ("英语", "#7FA3C7", false),
            ("数学", "#8DB59A", false),
            ("专业课", "#C7AE7F", false),
            ("其他", "#A3A9B4", true),
        ]
    );
}

#[test]
fn reopening_keeps_presets_without_duplicating() {
    let (dir, store) = open();
    drop(store);
    let store = reopen(&dir);
    assert_eq!(names(&store, true).len(), 5);
}

#[test]
fn created_subject_is_read_back_from_disk_just_before_other() {
    let (dir, store) = open();
    let created = store.write(|c| subjects::create(c, "计组", "#123456")).unwrap();
    drop(store);

    let store = reopen(&dir);
    let list = store.read(|c| subjects::list(c, false)).unwrap();
    let new = &list[list.len() - 2];
    assert_eq!((new.id, new.name.as_str(), new.color.as_str()), (created.id, "计组", "#123456"));
    assert!(!new.is_fallback);
    assert_eq!(names(&store, false), ["政治", "英语", "数学", "专业课", "计组", "其他"]);
}

#[test]
fn created_subject_goes_last_when_other_was_moved_up() {
    let (_dir, store) = open();
    let ids: Vec<i64> = ["其他", "政治", "英语", "数学", "专业课"].iter().map(|n| id_of(&store, n)).collect();
    store.write(|c| subjects::reorder(c, &ids)).unwrap();

    store.write(|c| subjects::create(c, "计组", "#123456")).unwrap();
    assert_eq!(names(&store, false), ["其他", "政治", "英语", "数学", "专业课", "计组"]);
}

#[test]
fn rename_and_recolor_are_persisted() {
    let (dir, store) = open();
    let id = id_of(&store, "专业课");
    store
        .write(|c| subjects::update(c, id, Some("数据结构"), Some("#AABBCC")))
        .unwrap();
    drop(store);

    let store = reopen(&dir);
    let s = store.read(|c| subjects::get(c, id)).unwrap();
    assert_eq!((s.name.as_str(), s.color.as_str()), ("数据结构", "#AABBCC"));
}

#[test]
fn name_is_trimmed_and_must_be_one_to_ten_characters() {
    let (_dir, store) = open();
    assert_invalid(store.write(|c| subjects::create(c, "   ", "#123456")), "名称");
    assert_invalid(store.write(|c| subjects::create(c, "一二三四五六七八九十一", "#123456")), "10");
    let s = store.write(|c| subjects::create(c, "  一二三四五六七八九十 ", "#123456")).unwrap();
    assert_eq!(s.name, "一二三四五六七八九十");
}

#[test]
fn name_cannot_repeat_an_existing_subject_even_if_archived() {
    let (_dir, store) = open();
    let politics = id_of(&store, "政治");
    store.write(|c| subjects::archive(c, politics)).unwrap();
    assert_invalid(store.write(|c| subjects::create(c, "政治", "#123456")), "已经有");
    let english = id_of(&store, "英语");
    assert_invalid(store.write(|c| subjects::update(c, english, Some("数学"), None)), "已经有");
    assert_eq!(names(&store, true).len(), 5);
}

#[test]
fn renaming_to_its_own_name_is_allowed() {
    let (_dir, store) = open();
    let math = id_of(&store, "数学");
    store.write(|c| subjects::update(c, math, Some("数学"), Some("#000000"))).unwrap();
    assert_eq!(store.read(|c| subjects::get(c, math)).unwrap().color, "#000000");
}

#[test]
fn other_can_only_change_color() {
    let (_dir, store) = open();
    let other = id_of(&store, "其他");
    assert_invalid(store.write(|c| subjects::update(c, other, Some("杂项"), None)), "其他");
    assert_invalid(store.write(|c| subjects::archive(c, other)), "其他");
    store.write(|c| subjects::update(c, other, None, Some("#010203"))).unwrap();
    let s = store.read(|c| subjects::get(c, other)).unwrap();
    assert_eq!((s.name.as_str(), s.color.as_str(), s.archived), ("其他", "#010203", false));
}

#[test]
fn archived_subject_leaves_active_list_but_is_kept_and_restorable() {
    let (_dir, store) = open();
    let english = id_of(&store, "英语");
    store.write(|c| subjects::archive(c, english)).unwrap();
    assert_eq!(names(&store, false), vec!["政治", "数学", "专业课", "其他"]);
    assert!(store.read(|c| subjects::get(c, english)).unwrap().archived);

    store.write(|c| subjects::restore(c, english)).unwrap();
    assert_eq!(names(&store, false), vec!["政治", "英语", "数学", "专业课", "其他"]);
}

#[test]
fn reorder_follows_the_given_order() {
    let (_dir, store) = open();
    let ids: Vec<i64> = ["其他", "数学", "政治", "英语", "专业课"].iter().map(|n| id_of(&store, n)).collect();
    store.write(|c| subjects::reorder(c, &ids)).unwrap();
    assert_eq!(names(&store, false), vec!["其他", "数学", "政治", "英语", "专业课"]);
}

#[test]
fn reorder_with_an_unknown_subject_changes_nothing() {
    let (_dir, store) = open();
    let other = id_of(&store, "其他");
    assert_invalid(store.write(|c| subjects::reorder(c, &[other, 999])), "科目");
    assert_eq!(names(&store, false), vec!["政治", "英语", "数学", "专业课", "其他"]);
}

#[test]
fn color_must_be_a_hex_color() {
    let (_dir, store) = open();
    assert_invalid(store.write(|c| subjects::create(c, "计网", "red")), "颜色");
    assert_invalid(store.write(|c| subjects::create(c, "计网", "#12345")), "颜色");
}

#[test]
fn unknown_subject_is_reported() {
    let (_dir, store) = open();
    assert_invalid(store.read(|c| subjects::get(c, 999)), "科目");
    assert_invalid(store.write(|c| subjects::archive(c, 999)), "科目");
}
