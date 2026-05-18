use assert_cmd::Command;

const DIGEST_FIXTURE: &str = "tests/fixtures/digest_tree";
const GO_FIXTURE: &str = "tests/fixtures/source_shapes.go";
const JS_FIXTURE: &str = "tests/fixtures/source_shapes.js";
const JSX_FIXTURE: &str = "tests/fixtures/source_shapes.jsx";
const MARKDOWN_FIXTURE: &str = "tests/fixtures/duplicate_headings.md";
const NO_BLOCK_PLS_FIXTURE: &str = "tests/fixtures/no_block_pls_shapes.rs";
const PARSE_ERROR_FIXTURE: &str = "tests/fixtures/parse_error.rs";
const PYTHON_FIXTURE: &str = "tests/fixtures/source_shapes.py";
const RUST_FIXTURE: &str = "tests/fixtures/rust_symbols.rs";
const TS_FIXTURE: &str = "tests/fixtures/source_shapes.ts";
const TSX_FIXTURE: &str = "tests/fixtures/source_shapes.tsx";
const UNSUPPORTED_FIXTURE: &str = "tests/fixtures/not_source.txt";

#[test]
fn map_prints_exact_keys_that_show_accepts() {
    let stdout = run_lupa(&["map", RUST_FIXTURE]);

    for key in [
        "key=Alpha\n",
        "key=Alpha.new\n",
        "key=Alpha.greet\n",
        "key=Beta\n",
        "key=Beta.new\n",
        "key=parse_config\n",
    ] {
        assert_stdout_contains(&stdout, key);
    }

    let stdout = run_lupa(&[
        "show",
        RUST_FIXTURE,
        "Alpha.new",
        "Alpha.greet",
        "Beta.new",
        "parse_config",
    ]);

    for key in [
        "key=Alpha.new",
        "key=Alpha.greet",
        "key=Beta.new",
        "key=parse_config",
    ] {
        assert_stdout_contains(&stdout, key);
    }
}

#[test]
fn show_accepts_multiple_keys_and_prints_plain_line_prefixes() {
    let stdout = run_lupa(&["show", RUST_FIXTURE, "Alpha.new", "Beta.new"]);

    for line in [
        "# tests/fixtures/rust_symbols.rs L6-L8 key=Alpha.new kind=method\n",
        "6|    pub fn new(name: String) -> Self {\n",
        "7|        Self { name }\n",
        "8|    }\n",
        "# tests/fixtures/rust_symbols.rs L18-L20 key=Beta.new kind=method\n",
        "18|    pub fn new() -> Self {\n",
        "19|        Self\n",
        "20|    }\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }
}

#[test]
fn ambiguous_suffix_reports_all_candidates() {
    let stdout = run_lupa(&["show", RUST_FIXTURE, "new"]);

    for line in [
        "# error: ambiguous key `new` in tests/fixtures/rust_symbols.rs\n",
        "# matches:\n",
        "key=Alpha.new\n",
        "key=Beta.new\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }

    assert_stdout_lacks(&stdout, "6|    pub fn new");
    assert_stdout_lacks(&stdout, "18|    pub fn new");
}

#[test]
fn markdown_duplicate_headings_get_deterministic_keys() {
    let stdout = run_lupa(&["map", MARKDOWN_FIXTURE]);
    let repeated_stdout = run_lupa(&["map", MARKDOWN_FIXTURE]);

    assert_stdout_contains(&stdout, "key=Install\n");
    assert_stdout_contains(&stdout, "key=Install#2\n");
    assert_eq!(stdout, repeated_stdout);
}

#[test]
fn keys_prints_key_range_lines() {
    let stdout = run_lupa(&["keys", RUST_FIXTURE]);

    for line in [
        "Alpha L1-L3\n",
        "Alpha.new L6-L8\n",
        "Alpha.greet L10-L12\n",
        "Beta L15\n",
        "Beta.new L18-L20\n",
        "parse_config L23-L25\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }

    assert_stdout_lacks(&stdout, "key=");
}

#[test]
fn digest_skips_ignored_directories() {
    let stdout = run_lupa(&["digest", DIGEST_FIXTURE]);

    assert_stdout_contains(&stdout, "tests/fixtures/digest_tree/visible.rs");
    assert_stdout_lacks(&stdout, "target/ignored.rs");
    assert_stdout_lacks(&stdout, "ignored.rs [rust]");
}

#[test]
fn digest_includes_polyglot_source_extensions() {
    let stdout = run_lupa(&["digest", DIGEST_FIXTURE]);

    for path in [
        "tests/fixtures/digest_tree/visible.go",
        "tests/fixtures/digest_tree/visible.js",
        "tests/fixtures/digest_tree/visible.jsx",
        "tests/fixtures/digest_tree/visible.py",
        "tests/fixtures/digest_tree/visible.ts",
        "tests/fixtures/digest_tree/visible.tsx",
    ] {
        assert_stdout_contains(&stdout, path);
    }
}

#[test]
fn parse_error_warning_appears_with_partial_output() {
    let stdout = run_lupa(&["map", PARSE_ERROR_FIXTURE]);

    assert_stdout_contains(&stdout, "# tests/fixtures/parse_error.rs [rust]");
    assert_stdout_contains(
        &stdout,
        "# warning: parse error at L1: parse error in ERROR\n",
    );
}

#[test]
fn unsupported_file_type_is_recoverable_error() {
    let stdout = run_lupa(&["map", UNSUPPORTED_FIXTURE]);

    assert_eq!(
        stdout,
        "# error: unsupported file type: tests/fixtures/not_source.txt\n"
    );
}

#[test]
fn help_exits_successfully() {
    let stdout = run_lupa(&["--help"]);

    assert_stdout_contains(&stdout, "Usage: lupa <COMMAND>\n");
    assert_stdout_contains(&stdout, "Commands:\n");
}

#[test]
fn no_block_pls_shapes_map_to_stable_keys() {
    let stdout = run_lupa(&["map", NO_BLOCK_PLS_FIXTURE]);

    for key in [
        "key=Receiver.recv\n",
        "key=Broadcaster.run\n",
        "key=Storage.remove_outdated_states\n",
        "key=poll_impl\n",
    ] {
        assert_stdout_contains(&stdout, key);
    }
}

#[test]
fn no_block_pls_shapes_show_generic_impl_and_long_functions() {
    let stdout = run_lupa(&[
        "show",
        NO_BLOCK_PLS_FIXTURE,
        "Receiver.recv",
        "Storage.remove_outdated_states",
        "poll_impl",
    ]);

    for line in [
        "key=Receiver.recv kind=method\n",
        "async fn recv(&mut self) -> Option<T> {\n",
        "key=Storage.remove_outdated_states kind=method\n",
        "pub async fn remove_outdated_states(&self, mc_seqno: u32) -> Result<(), Error> {\n",
        "key=poll_impl kind=function\n",
        "fn poll_impl<'cx, Fut>(\n",
        "where\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }
}

#[test]
fn polyglot_map_prints_expected_keys() {
    for (fixture, keys) in [
        (
            PYTHON_FIXTURE,
            &[
                "key=Service\n",
                "key=Service.__init__\n",
                "key=Service.start\n",
                "key=build_service\n",
            ][..],
        ),
        (
            JS_FIXTURE,
            &[
                "key=Widget\n",
                "key=Widget.constructor\n",
                "key=Widget.render\n",
                "key=makeWidget\n",
            ][..],
        ),
        (JSX_FIXTURE, &["key=Card\n", "key=Shell\n"][..]),
        (
            TS_FIXTURE,
            &[
                "key=Repository\n",
                "key=Repository.get\n",
                "key=User\n",
                "key=UserService\n",
                "key=UserService.constructor\n",
                "key=UserService.load\n",
                "key=formatUser\n",
            ][..],
        ),
        (
            TSX_FIXTURE,
            &[
                "key=ButtonProps\n",
                "key=ButtonProps.label\n",
                "key=ButtonProps.onClick\n",
                "key=Button\n",
                "key=Toolbar\n",
            ][..],
        ),
        (
            GO_FIXTURE,
            &[
                "key=Server\n",
                "key=Server.name\n",
                "key=Server.Handler\n",
                "key=Server.clock\n",
                "key=Server.Start\n",
                "key=Handler\n",
                "key=Handler.Handle\n",
                "key=Handler.Close\n",
                "key=Clock\n",
                "key=Clock.Now\n",
                "key=Alias\n",
                "key=NewServer\n",
                "key=helper\n",
            ][..],
        ),
    ] {
        let stdout = run_lupa(&["map", fixture]);
        for key in keys {
            assert_stdout_contains(&stdout, key);
        }
    }
}

#[test]
fn polyglot_show_prints_selected_symbols() {
    for (fixture, keys, expected) in [
        (
            PYTHON_FIXTURE,
            &["Service.start", "build_service"][..],
            &[
                "# tests/fixtures/source_shapes.py L7-L8 key=Service.start kind=method\n",
                "7|    async def start(self, retries: int = 1) -> str:\n",
                "# tests/fixtures/source_shapes.py L10-L11 key=build_service kind=function\n",
                "10|def build_service(label: str) -> Service:\n",
            ][..],
        ),
        (
            JS_FIXTURE,
            &["Widget.render", "makeWidget"][..],
            &[
                "# tests/fixtures/source_shapes.js L6-L8 key=Widget.render kind=method\n",
                "6|    render(target) {\n",
                "# tests/fixtures/source_shapes.js L11-L13 key=makeWidget kind=function\n",
                "11|export function makeWidget(name) {\n",
            ][..],
        ),
        (
            JSX_FIXTURE,
            &["Card", "Shell"][..],
            &[
                "# tests/fixtures/source_shapes.jsx L1-L3 key=Card kind=function\n",
                "1|export function Card({ title }) {\n",
                "# tests/fixtures/source_shapes.jsx L5-L7 key=Shell kind=function\n",
                "5|export const Shell = () => {\n",
            ][..],
        ),
        (
            TS_FIXTURE,
            &["UserService.load", "formatUser"][..],
            &[
                "# tests/fixtures/source_shapes.ts L13-L15 key=UserService.load kind=method\n",
                "13|    async load(id: string): Promise<User> {\n",
                "# tests/fixtures/source_shapes.ts L18-L20 key=formatUser kind=function\n",
                "18|export function formatUser(user: User): string {\n",
            ][..],
        ),
        (
            TSX_FIXTURE,
            &["Button", "Toolbar"][..],
            &[
                "# tests/fixtures/source_shapes.tsx L6-L8 key=Button kind=function\n",
                "6|export function Button(props: ButtonProps) {\n",
                "# tests/fixtures/source_shapes.tsx L10-L12 key=Toolbar kind=function\n",
                "10|export const Toolbar = () => {\n",
            ][..],
        ),
        (
            GO_FIXTURE,
            &["Server.Start", "NewServer"][..],
            &[
                "# tests/fixtures/source_shapes.go L26-L28 key=Server.Start kind=method\n",
                "26|func (s *Server) Start(ctx context.Context) error {\n",
                "# tests/fixtures/source_shapes.go L22-L24 key=NewServer kind=function\n",
                "22|func NewServer(name string, handler Handler) *Server {\n",
            ][..],
        ),
    ] {
        let mut args = vec!["show", fixture];
        args.extend_from_slice(keys);
        let stdout = run_lupa(&args);
        for line in expected {
            assert_stdout_contains(&stdout, line);
        }
    }
}

#[test]
fn polyglot_keys_print_expected_ranges() {
    for (fixture, expected) in [
        (
            PYTHON_FIXTURE,
            &[
                "Service L1-L8\n",
                "Service.start L7-L8\n",
                "build_service L10-L11\n",
            ][..],
        ),
        (
            JS_FIXTURE,
            &[
                "Widget L1-L9\n",
                "Widget.render L6-L8\n",
                "makeWidget L11-L13\n",
            ][..],
        ),
        (JSX_FIXTURE, &["Card L1-L3\n", "Shell L5-L7\n"][..]),
        (
            TS_FIXTURE,
            &[
                "Repository L1-L3\n",
                "UserService.load L13-L15\n",
                "formatUser L18-L20\n",
            ][..],
        ),
        (
            TSX_FIXTURE,
            &["ButtonProps L1-L4\n", "Button L6-L8\n", "Toolbar L10-L12\n"][..],
        ),
        (
            GO_FIXTURE,
            &[
                "Server L5-L9\n",
                "Server.name L6\n",
                "Server.Handler L7\n",
                "Server.Start L26-L28\n",
                "Handler.Handle L12\n",
                "NewServer L22-L24\n",
            ][..],
        ),
    ] {
        let stdout = run_lupa(&["keys", fixture]);
        for line in expected {
            assert_stdout_contains(&stdout, line);
        }
        assert_stdout_lacks(&stdout, "key=");
    }
}

fn run_lupa(args: &[&str]) -> String {
    let output = Command::cargo_bin("lupa")
        .expect("lupa binary should build")
        .args(args)
        .output()
        .expect("lupa command should run");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(
        output.status.success(),
        "lupa {args:?} failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stderr, "", "lupa {args:?} wrote stderr");

    stdout
}

fn assert_stdout_contains(stdout: &str, needle: &str) {
    assert!(
        stdout.contains(needle),
        "stdout missing {needle:?}\nstdout:\n{stdout}"
    );
}

fn assert_stdout_lacks(stdout: &str, needle: &str) {
    assert!(
        !stdout.contains(needle),
        "stdout unexpectedly contained {needle:?}\nstdout:\n{stdout}"
    );
}
