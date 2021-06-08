use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    io,
};
use typescript_type_def::{Context, EmitDepsContext, TypescriptTypeDef};

fn test_emit<T>() -> String
where
    T: TypescriptTypeDef,
{
    let mut buf = Vec::new();
    let mut ctx = EmitDepsContext::new(&mut buf);
    ctx.emit_dep::<T>().unwrap();
    String::from_utf8(buf).unwrap()
}

macro_rules! assert_eq_str {
    ($actual:expr, $expected:expr) => {{
        let actual = $actual;
        let expected = $expected;
        assert_eq!(
            actual, expected,
            "strings differed:\nactual:\n{}\nexpected:\n{}",
            actual, expected
        );
    }};
}

#[test]
fn emit() {
    type Inner = Vec<HashMap<Option<usize>, HashSet<String>>>;

    #[derive(Serialize)]
    struct Test(Inner);

    impl TypescriptTypeDef for Test {
        fn emit_name(ctx: &mut Context<'_>) -> io::Result<()> {
            write!(ctx.out, "Test")
        }

        fn emit_deps(ctx: &mut EmitDepsContext<'_>) -> io::Result<()> {
            ctx.emit_dep::<Inner>()?;
            Ok(())
        }

        fn emit_def(ctx: &mut Context<'_>) -> io::Result<()> {
            write!(ctx.out, "export type {}=", stringify!(Test))?;
            ctx.emit_name::<Inner>()?;
            write!(ctx.out, ";")?;
            Ok(())
        }
    }

    assert_eq_str!(
        test_emit::<Test>(),
        r#"export namespace types{export type Usize=number;}export namespace types{export type Test=(Record<(types.Usize|null),Set<string>>[]);}"#
    );
}

#[test]
fn derive() {
    #![allow(dead_code)]

    #[derive(Serialize, TypescriptTypeDef)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    struct Parent {
        foo_bar: usize,
    }

    #[derive(Serialize, TypescriptTypeDef)]
    struct Test {
        #[serde(flatten)]
        parent: Parent,
        a: String,
        b: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        c: Option<Vec<bool>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        d: Option<u8>,
    }

    #[derive(Serialize, TypescriptTypeDef)]
    struct Test2(Test, usize, String);

    #[derive(Serialize, TypescriptTypeDef)]
    struct Test3(Test2);

    #[derive(Serialize, TypescriptTypeDef)]
    #[serde(untagged)]
    enum Test4 {
        A(Test3),
        B(String, usize),
        C { a: String, b: usize },
    }

    #[derive(Serialize, TypescriptTypeDef)]
    #[serde(rename_all = "kebab-case")]
    enum Test5 {
        A,
        B,
        CoolBeans,
    }

    #[derive(Serialize, TypescriptTypeDef)]
    enum Test6 {
        A { a: usize },
        B(usize, String),
        C(String),
        D,
    }

    #[derive(Serialize, TypescriptTypeDef)]
    #[serde(tag = "type", content = "value")]
    enum Test7 {
        A {
            a: String,
            b: usize,
        },
        #[serde(rename_all = "UPPERCASE")]
        B {
            a: Test4,
            b: Test5,
            c: Test6,
        },
        C(Parent),
        D,
    }

    macro_rules! assert_eq_str {
        ($actual:expr, $expected:expr) => {{
            let actual = $actual;
            let expected = $expected;
            assert_eq!(
                actual, expected,
                "strings differed:\nactual:\n{}\nexpected:\n{}",
                actual, expected
            );
        }};
    }

    assert_eq_str!(
        test_emit::<Test7>(),
        r#"export namespace types{export type Usize=number;}export namespace types{export type Parent={"FOO_BAR":types.Usize;};}export namespace types{export type U8=number;}export namespace types{export type Test=types.Parent&{"a":string;"b":(types.Usize|null);"c"?:(boolean[]);"d"?:types.U8;};}export namespace types{export type Test2=[types.Test,types.Usize,string];}export namespace types{export type Test3=types.Test2;}export namespace types{export type Test4=|types.Test3|[string,types.Usize]|{"a":string;"b":types.Usize;};}export namespace types{export type Test5=|"a"|"b"|"cool-beans";}export namespace types{export type Test6=|{"A":{"a":types.Usize;}}|{"B":[types.Usize,string];}|{"C":string;}|"D";}export namespace types{export type Test7=|{"type":"A";"value":{"a":string;"b":types.Usize;};}|{"type":"B";"value":{"A":types.Test4;"B":types.Test5;"C":types.Test6;};}|{"type":"C";"value":types.Parent;}|{"type":"D";};}"#
    );

    assert_eq_str!(
        serde_json::to_string(&Test7::B {
            a: Test4::A(Test3(Test2(
                Test {
                    parent: Parent {
                        foo_bar: 123
                    },
                    a: "foo".to_owned(),
                    b: None,
                    c: Some(vec![true, false]),
                    d: None,
                },
                4,
                "bar".to_owned(),
            ))),
            b: Test5::CoolBeans,
            c: Test6::B(42, "baz".to_owned()),
        })
        .unwrap(),
        r#"{"type":"B","value":{"A":[{"FOO_BAR":123,"a":"foo","b":null,"c":[true,false]},4,"bar"],"B":"cool-beans","C":{"B":[42,"baz"]}}}"#
    );
}

#[test]
fn enum_tags() {
    #![allow(dead_code)]

    #[derive(Clone, Copy, Serialize, TypescriptTypeDef)]
    struct Inner {
        x: bool,
    }

    #[derive(Serialize, TypescriptTypeDef)]
    #[serde(tag = "type")]
    enum Test1 {
        A { a: Inner },
        B(Inner),
        // C(Inner, Inner), // not allowed
        D,
    }

    #[derive(Serialize, TypescriptTypeDef)]
    #[serde(tag = "type", content = "value")]
    enum Test2 {
        A { a: Inner },
        B(Inner),
        // C(Inner, Inner), // not allowed
        D,
    }

    #[derive(Serialize, TypescriptTypeDef)]
    #[serde(untagged)]
    enum Test3 {
        A { a: Inner },
        B(Inner),
        C(Inner, Inner),
        D,
    }

    #[derive(Serialize, TypescriptTypeDef)]
    enum Test4 {
        A { a: Inner },
        B(Inner),
        C(Inner, Inner),
        D,
    }

    let inner = Inner { x: true };

    assert_eq_str!(
        test_emit::<Test1>(),
        r#"export namespace types{export type Inner={"x":boolean;};}export namespace types{export type Test1=|{"type":"A";"a":types.Inner;}|(types.Inner&{"type":"B";})|{"type":"D";};}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test1::A { a: inner }).unwrap(),
        r#"{"type":"A","a":{"x":true}}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test1::B(inner)).unwrap(),
        r#"{"type":"B","x":true}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test1::D).unwrap(),
        r#"{"type":"D"}"#
    );

    assert_eq_str!(
        test_emit::<Test2>(),
        r#"export namespace types{export type Inner={"x":boolean;};}export namespace types{export type Test2=|{"type":"A";"value":{"a":types.Inner;};}|{"type":"B";"value":types.Inner;}|{"type":"D";};}"#);
    assert_eq_str!(
        serde_json::to_string(&Test2::A { a: inner }).unwrap(),
        r#"{"type":"A","value":{"a":{"x":true}}}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test2::B(inner)).unwrap(),
        r#"{"type":"B","value":{"x":true}}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test2::D).unwrap(),
        r#"{"type":"D"}"#
    );

    assert_eq_str!(
        test_emit::<Test3>(),
        r#"export namespace types{export type Inner={"x":boolean;};}export namespace types{export type Test3=|{"a":types.Inner;}|types.Inner|[types.Inner,types.Inner]|null;}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test3::A { a: inner }).unwrap(),
        r#"{"a":{"x":true}}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test3::B(inner)).unwrap(),
        r#"{"x":true}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test3::C(inner, inner)).unwrap(),
        r#"[{"x":true},{"x":true}]"#
    );
    assert_eq_str!(serde_json::to_string(&Test3::D).unwrap(), r#"null"#);

    assert_eq_str!(
        test_emit::<Test4>(),
        r#"export namespace types{export type Inner={"x":boolean;};}export namespace types{export type Test4=|{"A":{"a":types.Inner;}}|{"B":types.Inner;}|{"C":[types.Inner,types.Inner];}|"D";}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test4::A { a: inner }).unwrap(),
        r#"{"A":{"a":{"x":true}}}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test4::B(inner)).unwrap(),
        r#"{"B":{"x":true}}"#
    );
    assert_eq_str!(
        serde_json::to_string(&Test4::C(inner, inner)).unwrap(),
        r#"{"C":[{"x":true},{"x":true}]}"#
    );
    assert_eq_str!(serde_json::to_string(&Test4::D).unwrap(), r#""D""#);
}
