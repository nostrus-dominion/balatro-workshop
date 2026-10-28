use balatro_workshop::{
    codec, discovery,
    document::{create_file, Document},
    lua::{self, Key, Value},
};
use flate2::{write::DeflateEncoder, Compression};
use std::{fs, io::Write};

fn parse(s: &str) -> Value {
    lua::parse(s.as_bytes()).unwrap()
}
fn fixture() -> Value {
    parse(include_str!("fixtures/run.lua"))
}

#[test]
fn independent_zlib_fixture_loads() {
    let decoded = codec::decode(include_bytes!("fixtures/run.jkr")).unwrap();
    assert_eq!(decoded, fixture());
    assert_eq!(
        decoded.get("GAME.dollars"),
        Some(&Value::Number("125".into()))
    );
}

#[test]
fn encode_decode_preserves_every_value() {
    let value = fixture();
    assert_eq!(
        codec::decode(&codec::encode(&value).unwrap()).unwrap(),
        value
    );
}

#[test]
fn mixed_sparse_keys_remain_distinct() {
    let v = parse(
        r#"return {[1]="one", [9]="nine", ["1"]="string", [0]=false, [-2.5]=true, [true]="boolean"}"#,
    );
    assert_eq!(parse(&lua::serialize(&v)), v);
    let Value::Table(entries) = v else { panic!() };
    assert!(matches!(&entries[0].0, Key::Number(n) if n == "1"));
    assert!(matches!(&entries[2].0, Key::String(n) if n == b"1"));
}

#[test]
fn empty_tables_stay_tables() {
    let v = parse("return {cards={},tags={},nested={a={}}}");
    assert_eq!(v.get("cards"), Some(&Value::Table(Vec::new())));
    assert_eq!(parse(&lua::serialize(&v)), v);
}

#[test]
fn escaped_and_binary_strings_round_trip() {
    let v = parse(
        r#"return {message="return [\"x\"]=,} \\ \000\255\0107", ['quote']= 'it\'s okay', unicode="♥ 日本語"}"#,
    );
    assert_eq!(parse(&lua::serialize(&v)), v);
}

#[test]
fn large_numbers_keep_original_lexemes() {
    let v = parse("return {a=9007199254740993,b=1.2345678901234567e100,c=-0.00001,d=1E+09}");
    assert_eq!(v.get("a"), Some(&Value::Number("9007199254740993".into())));
    assert_eq!(parse(&lua::serialize(&v)), v);
}

#[test]
fn comments_long_strings_and_positional_values() {
    let v = parse("-- comment\nreturn {true, false; --[[ skip ]] text=[=[\nhello\r\nworld]=], more={7,8,9},}; -- done");
    assert_eq!(
        v.get("text"),
        Some(&Value::String(b"hello\nworld".to_vec()))
    );
    assert_eq!(parse(&lua::serialize(&v)), v);
}

#[test]
fn decimal_escape_uses_at_most_three_digits() {
    let v = parse(r#"return {s="\0012\12\x41\z  \n"}"#);
    assert_eq!(
        v.get("s"),
        Some(&Value::String(vec![1, b'2', 12, b'A', b'\n']))
    );
}

#[test]
fn malformed_or_executable_lua_is_rejected() {
    for text in [
        "return os.execute('anything')",
        "return {x=function() end}",
        "return {} os.execute('anything')",
        "return {x=1+2}",
        "return {x=0/0}",
        "return {x=math.huge}",
        "return {x=1e999}",
        "return {x=NaN}",
        "return {x=nil}",
        "return {['a']=1",
        "return {x='unterminated}",
        "return {true=1}",
        "return {x=0xFF}",
        "return {x='\\256'}",
        "return 1",
    ] {
        assert!(
            lua::parse(text.as_bytes()).is_err(),
            "Unexpectedly accepted: {text}"
        );
    }
}

#[test]
fn duplicate_keys_and_numeric_aliases_rejected() {
    for text in [
        "return {a=1,a=2}",
        "return {[1]=1,[1.0]=2}",
        "return {[0]=1,[-0]=2}",
        "return {7,[1]=2}",
    ] {
        assert!(lua::parse(text.as_bytes()).is_err());
    }
}

#[test]
fn nesting_limit_is_enforced() {
    let input = format!("return {}true{}", "{a=".repeat(130), "}".repeat(130));
    assert!(lua::parse(input.as_bytes()).unwrap_err().contains("limits"));
}

#[test]
fn truncation_and_trailing_data_are_rejected() {
    let bytes = codec::encode(&fixture()).unwrap();
    for n in 0..bytes.len() {
        assert!(
            codec::decode(&bytes[..n]).is_err(),
            "Truncation {n} accepted"
        );
    }
    let mut extra = bytes;
    extra.push(0);
    assert!(codec::decode(&extra).is_err());
    assert!(codec::decode(b"not a save").is_err());
}

#[test]
fn decompression_bomb_is_bounded() {
    let mut compressor = DeflateEncoder::new(Vec::new(), Compression::fast());
    let block = [b' '; 8192];
    for _ in 0..(lua::MAX_TEXT / block.len() + 1) {
        compressor.write_all(&block).unwrap();
    }
    let error = codec::decode(&compressor.finish().unwrap()).unwrap_err();
    assert!(error.contains("32 MiB"));
}

#[test]
fn invalid_edit_never_touches_source() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.jkr");
    let original = codec::encode(&fixture()).unwrap();
    fs::write(&path, &original).unwrap();
    let mut doc = Document::open(&path).unwrap();
    *doc.data.get_mut("GAME.dollars").unwrap() = Value::Number("1+2".into());
    assert!(doc.save().is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn save_creates_exact_backup_and_valid_output() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.jkr");
    let original = codec::encode(&fixture()).unwrap();
    fs::write(&path, &original).unwrap();
    let mut doc = Document::open(&path).unwrap();
    *doc.data.get_mut("GAME.dollars").unwrap() = Value::Number("999".into());
    assert!(doc.dirty());
    let backup = doc.save().unwrap();
    assert!(!doc.dirty());
    assert_eq!(fs::read(backup).unwrap(), original);
    assert_eq!(
        Document::open(&path).unwrap().data.get("GAME.dollars"),
        Some(&Value::Number("999".into()))
    );
    assert_eq!(
        Document::open(&path).unwrap().data.get("unknown_mod_data"),
        fixture().get("unknown_mod_data")
    );
}

#[test]
fn repeated_saves_have_distinct_backups() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.jkr");
    fs::write(&path, codec::encode(&fixture()).unwrap()).unwrap();
    let mut doc = Document::open(&path).unwrap();
    let first = doc.save().unwrap();
    let second = doc.save().unwrap();
    assert_ne!(first, second);
    assert!(first.exists() && second.exists());
}

#[test]
fn external_modification_is_not_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.jkr");
    fs::write(&path, codec::encode(&fixture()).unwrap()).unwrap();
    let mut doc = Document::open(&path).unwrap();
    let replacement = codec::encode(&parse("return {changed=true}")).unwrap();
    fs::write(&path, &replacement).unwrap();
    assert!(doc.save().unwrap_err().contains("changed on disk"));
    assert_eq!(fs::read(&path).unwrap(), replacement);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn save_copy_preserves_original_and_refuses_collision() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.jkr");
    let original = codec::encode(&fixture()).unwrap();
    fs::write(&path, &original).unwrap();
    let mut doc = Document::open(&path).unwrap();
    *doc.data.get_mut("GAME.dollars").unwrap() = Value::Number("500".into());
    assert!(doc.save_copy(&path).is_err());
    let target = dir.path().join("copy.jkr");
    doc.save_copy(&target).unwrap();
    assert_eq!(fs::read(&path).unwrap(), original);
    assert!(!doc.dirty());
    assert_eq!(doc.path, fs::canonicalize(target).unwrap());
}

#[test]
fn revert_restores_loaded_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.jkr");
    fs::write(&path, codec::encode(&fixture()).unwrap()).unwrap();
    let mut doc = Document::open(&path).unwrap();
    doc.data = parse("return {}");
    assert!(doc.dirty());
    doc.revert();
    assert!(!doc.dirty());
    assert_eq!(doc.data, fixture());
}

#[test]
fn export_refuses_existing_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.lua");
    create_file(&path, b"original").unwrap();
    assert!(create_file(&path, b"replacement").is_err());
    assert_eq!(fs::read(path).unwrap(), b"original");
}

#[test]
fn steam_external_library_paths() {
    let paths = discovery::library_paths("\"libraryfolders\"\n{\n \"path\" \"/mnt/Games/SteamLibrary\"\n \"path\" \"D:\\\\SteamLibrary\"\n}");
    assert_eq!(paths[0].to_string_lossy(), "/mnt/Games/SteamLibrary");
    assert_eq!(paths[1].to_string_lossy(), "D:\\SteamLibrary");
}

#[test]
fn discovery_finds_profiles_and_ignores_backups() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("1")).unwrap();
    fs::write(dir.path().join("1/save.jkr"), []).unwrap();
    fs::write(dir.path().join("settings.jkr"), []).unwrap();
    fs::write(dir.path().join("1/save.jkr.backup-abc"), []).unwrap();
    assert_eq!(discovery::scan(dir.path()).len(), 2);
}

#[test]
fn numeric_validation_is_strict() {
    for s in ["0", "-1", "1e+20", "1E-3", ".5", "5.", "9007199254740993"] {
        assert!(lua::valid_number(s));
    }
    for s in [
        "", "-", "NaN", "inf", "1e", "1+2", "1e999", "1.2.3", " 3", "+4",
    ] {
        assert!(!lua::valid_number(s));
    }
}

#[cfg(unix)]
#[test]
fn file_permissions_preserved() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.jkr");
    fs::write(&path, codec::encode(&fixture()).unwrap()).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    Document::open(&path).unwrap().save().unwrap();
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o640
    );
}
