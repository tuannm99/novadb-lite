use novadb_storage::{
    FilePager, Pager, SlottedPage, PAGE_SIZE, PAGE_TYPE_HEAP, SLOTTED_HEADER_SIZE,
    SLOTTED_SLOT_SIZE,
};

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "novadb-storage-integration-{name}-{}-{unique}.db",
        std::process::id()
    ))
}

#[test]
fn slotted_page_roundtrip_via_public_api() {
    let mut buf = vec![0; PAGE_SIZE];
    let mut page = SlottedPage::new(&mut buf).unwrap();
    page.init(PAGE_TYPE_HEAP).unwrap();

    let id0 = page.insert(b"alpha").unwrap();
    let id1 = page.insert(b"beta-record").unwrap();
    assert_eq!(0, id0);
    assert_eq!(1, id1);
    assert_eq!(Some("alpha".as_bytes()), page.get(id0).unwrap());
    assert_eq!(Some("beta-record".as_bytes()), page.get(id1).unwrap());
    assert_eq!(
        (PAGE_SIZE - SLOTTED_HEADER_SIZE - 2 * SLOTTED_SLOT_SIZE - 5 - 11) as u16,
        page.free_space().unwrap()
    );

    let moved = page
        .update(id0, b"alpha record moved to new position")
        .unwrap();
    assert!(moved);
    page.delete(id1).unwrap();
    assert_eq!(
        Some("alpha record moved to new position".as_bytes()),
        page.get(id0).unwrap()
    );
    assert_eq!(None, page.get(id1).unwrap());
    page.validate_full().unwrap();
}

#[test]
fn file_pager_persists_page_bytes() {
    let path = temp_db_path("persist");
    let mut pager = FilePager::open(&path).unwrap();
    let pid = pager.alloc_page().unwrap();

    let mut page_buf = vec![0; PAGE_SIZE];
    {
        let mut page = SlottedPage::new(&mut page_buf).unwrap();
        page.init(PAGE_TYPE_HEAP).unwrap();
        page.insert(b"persisted-record").unwrap();
    }

    pager.write_page(pid, &page_buf).unwrap();
    pager.flush().unwrap();

    let mut out = vec![0; PAGE_SIZE];
    pager.read_page(pid, &mut out).unwrap();
    let page = SlottedPage::new(&mut out).unwrap();
    assert_eq!(Some("persisted-record".as_bytes()), page.get(0).unwrap());

    let _ = std::fs::remove_file(path);
}
