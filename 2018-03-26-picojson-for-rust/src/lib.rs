/*

TODOs:

- [ ] parse string
    - [ ] use Read
    - [ ] support file stream
    - [ ] \u+FFFF
    - [ ] \b, \f
- [ ] serialize
    - [ ] \u+FFFF
- [ ] i64 support
- [ ] methods of Value
    - [ ] insert
    - [ ] erase
- [ ] refactoring

*/

use std::collections::BTreeMap;
use std::iter::FromIterator;
use std::iter::*;
use std::vec::*;

// static indent_width: i32 = 2;

type Error = String;

pub type Array = Vec<Value>;

pub type Object = BTreeMap<String, Value>;

/// Represents a reference to a key of json-like value.
#[derive(PartialEq, PartialOrd, Clone, Hash, Debug)]
pub enum ValueKey<'a> {
    Index(usize),
    Key(&'a str),
}

impl<'a> From<usize> for ValueKey<'a> {
    fn from(value: usize) -> ValueKey<'a> {
        ValueKey::Index(value)
    }
}

impl<'a> From<&'a str> for ValueKey<'a> {
    fn from(key: &'a str) -> Self {
        ValueKey::Key(key)
    }
}

#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub enum Value {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Array),
    Object(Object),
}

macro_rules! impl_value_as {
    ($n:ident, $m:ident, $t:ty) => {
        pub fn $n(&self) -> Option<&$t> {
            self.try_as::<$t>()
        }

        pub fn $m(&mut self) -> Option<&mut $t> {
            self.try_as_mut::<$t>()
        }
    }
}

impl Value {
    pub fn array() -> Value {
        Value::Array(Vec::new())
    }

    pub fn object() -> Value {
        Value::Object(BTreeMap::new())
    }

    fn from<T: ValueLike>(value: T) -> Value {
        value.into_value()
    }

    fn try_from_number(value: f64) -> Result<Value, &'static str> {
        if value.is_nan() {
            Err("NaN can't be a json value.")
        } else if value.is_infinite() {
            Err("Infinite value can't be a json value.")
        } else {
            Ok(Value::Number(value))
        }
    }

    fn from_number(value: f64) -> Value {
        Value::try_from_number(value).unwrap()
    }

    pub fn try_as<T: ValueKind>(&self) -> Option<&T> {
        T::try_borrow(self)
    }

    pub fn try_as_mut<T: ValueKind>(&mut self) -> Option<&mut T> {
        T::try_borrow_mut(self)
    }

    pub fn is_of<T: ValueKind>(&self) -> bool {
        self.try_as::<T>().is_some()
    }

    pub fn is_null(&self) -> bool {
        self.is_of::<()>()
    }

    impl_value_as!(as_bool, as_bool_mut, bool);
    impl_value_as!(as_number, as_number_mut, f64);
    impl_value_as!(as_string, as_string_mut, String);
    impl_value_as!(as_array, as_array_mut, Array);
    impl_value_as!(as_object, as_object_mut, Object);

    pub fn serialize(&self) -> String {
        let mut buf = Vec::<u8>::new();
        serialize(self, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    pub fn pretty_print(&self) -> String {
        const DEFAULT_INDENT_WIDTH: i32 = 2;

        let mut buf = Vec::<u8>::new();
        pretty_print(self, DEFAULT_INDENT_WIDTH, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    /// Determines if the value is empty, i.e., an empty array or object.
    pub fn is_empty(&self) -> bool {
        match *self {
            Value::Null | Value::Boolean(_) | Value::Number(_) | Value::String(_) => false,
            Value::Array(ref array) => array.is_empty(),
            Value::Object(ref object) => object.is_empty(),
        }
    }

    /// Determines if the value is a collection and has item for the specified key.
    pub fn has<'a, K: Into<ValueKey<'a>>>(&self, key: K) -> bool {
        self.get(key).is_some()
    }

    /// Gets an item for the specified key in the value if it's a collection.
    pub fn get<'a, K: Into<ValueKey<'a>>>(&self, key: K) -> Option<&Value> {
        match (self, key.into()) {
            (&Value::Array(ref array), ValueKey::Index(index)) => array.get(index),
            (&Value::Object(ref object), ValueKey::Key(key)) => object.get(key),
            _ => None,
        }
    }

    /// Gets a mutable reference to an item for the specified key in the value if it's a collection.
    pub fn get_mut<'a, K: Into<ValueKey<'a>>>(&mut self, key: K) -> Option<&mut Value> {
        match (self, key.into()) {
            (&mut Value::Array(ref mut array), ValueKey::Index(index)) => array.get_mut(index),
            (&mut Value::Object(ref mut object), ValueKey::Key(key)) => object.get_mut(key),
            _ => None,
        }
    }

    /// Adds a value to an array.
    pub fn push(&mut self, value: Value) {
        self.as_array_mut().map(|a| a.push(value));
    }

    /// Inserts an item to an array or object.
    /// Returns the old value if it's an object and has an item for the specified key.
    pub fn insert<'a, K: Into<ValueKey<'a>>>(&mut self, key: K, value: Value) -> Option<Value> {
        match (self, key.into()) {
            (&mut Value::Array(ref mut array), ValueKey::Index(index)) => {
                array.insert(index, value);
                None
            }
            (&mut Value::Object(ref mut object), ValueKey::Key(key)) => {
                object.insert(key.to_string(), value)
            }
            _ => None,
        }
    }

    /// Removes an item from an array or object.
    /// Returns the removed value if success.
    pub fn remove<'a, K: Into<ValueKey<'a>>>(&mut self, key: K) -> Option<Value> {
        match (self, key.into()) {
            (&mut Value::Array(ref mut array), ValueKey::Index(index)) => Some(array.remove(index)),
            (&mut Value::Object(ref mut object), ValueKey::Key(key)) => object.remove(key),
            _ => None,
        }
    }
}

pub trait ValueKind: Clone + std::fmt::Debug + ValueLike {
    fn try_borrow(value: &Value) -> Option<&Self>;

    fn try_borrow_mut(value: &mut Value) -> Option<&mut Self>;

    fn into_value(self) -> Value {
        <Self as ValueLike>::into_value(self)
    }
}

impl ValueKind for () {
    fn try_borrow(value: &Value) -> Option<&Self> {
        match value {
            &Value::Null => Some(&()),
            _ => None,
        }
    }

    fn try_borrow_mut(value: &mut Value) -> Option<&mut Self> {
        static mut UNIT: () = ();
        match value {
            &mut Value::Null => Some(unsafe { &mut UNIT }),
            _ => None,
        }
    }
}

impl ValueKind for bool {
    fn try_borrow(value: &Value) -> Option<&Self> {
        match value {
            &Value::Boolean(ref it) => Some(it),
            _ => None,
        }
    }

    fn try_borrow_mut(value: &mut Value) -> Option<&mut Self> {
        match value {
            &mut Value::Boolean(ref mut it) => Some(it),
            _ => None,
        }
    }
}

impl ValueKind for f64 {
    fn try_borrow(value: &Value) -> Option<&Self> {
        match value {
            &Value::Number(ref it) => Some(it),
            _ => None,
        }
    }

    fn try_borrow_mut(value: &mut Value) -> Option<&mut Self> {
        match value {
            &mut Value::Number(ref mut it) => Some(it),
            _ => None,
        }
    }
}

impl ValueKind for String {
    fn try_borrow(value: &Value) -> Option<&Self> {
        match value {
            &Value::String(ref it) => Some(it),
            _ => None,
        }
    }

    fn try_borrow_mut(value: &mut Value) -> Option<&mut Self> {
        match value {
            &mut Value::String(ref mut it) => Some(it),
            _ => None,
        }
    }
}

impl ValueKind for Array {
    fn try_borrow(value: &Value) -> Option<&Self> {
        match value {
            &Value::Array(ref it) => Some(it),
            _ => None,
        }
    }

    fn try_borrow_mut(value: &mut Value) -> Option<&mut Self> {
        match value {
            &mut Value::Array(ref mut it) => Some(it),
            _ => None,
        }
    }
}

impl ValueKind for Object {
    fn try_borrow(value: &Value) -> Option<&Self> {
        match value {
            &Value::Object(ref it) => Some(it),
            _ => None,
        }
    }

    fn try_borrow_mut(value: &mut Value) -> Option<&mut Self> {
        match value {
            &mut Value::Object(ref mut it) => Some(it),
            _ => None,
        }
    }
}

/// Represents a type of JSON value.
pub trait ValueLike: Clone + std::fmt::Debug {
    fn into_value(self) -> Value;
}

impl ValueLike for Value {
    fn into_value(self) -> Value {
        self
    }
}

impl ValueLike for () {
    fn into_value(self) -> Value {
        Value::Null
    }
}

impl ValueLike for bool {
    fn into_value(self) -> Value {
        Value::Boolean(self)
    }
}

impl ValueLike for f64 {
    fn into_value(self) -> Value {
        Value::from_number(self)
    }
}

impl ValueLike for i32 {
    fn into_value(self) -> Value {
        Value::Number(self as f64)
    }
}

impl ValueLike for String {
    fn into_value(self) -> Value {
        Value::String(self)
    }
}

fn array_from_iter<V, I>(iter: I) -> Array
where
    V: ValueLike,
    I: IntoIterator<Item = V>,
{
    iter.into_iter().map(|v| v.into_value()).collect::<Array>()
}

fn object_from_iter<K, V, I>(iter: I) -> Object
where
    K: ToString,
    V: ValueLike,
    I: IntoIterator<Item = (K, V)>,
{
    iter.into_iter()
        .map(|(k, v)| (k.to_string(), v.into_value()))
        .collect::<BTreeMap<_, _>>()
}

pub trait ValueCollectionItem: Clone + std::fmt::Debug {
    fn collect<I: IntoIterator<Item = Self>>(iter: I) -> Value;
}

impl<V: ValueLike> ValueCollectionItem for V {
    fn collect<I: IntoIterator<Item = V>>(iter: I) -> Value {
        Value::Array(array_from_iter(iter))
    }
}

impl<K, V> ValueCollectionItem for (K, V)
where
    K: ToString + Clone + std::fmt::Debug,
    V: ValueLike,
{
    fn collect<I: IntoIterator<Item = Self>>(iter: I) -> Value {
        Value::Object(object_from_iter(iter))
    }
}

impl<'a> ValueLike for &'a str {
    fn into_value(self) -> Value {
        Value::String(self.to_string())
    }
}

impl<T: ValueCollectionItem> ValueLike for Vec<T>
where
    T: Clone + std::fmt::Debug,
{
    fn into_value(self) -> Value {
        <T as ValueCollectionItem>::collect(self)
    }
}

impl<K, V> ValueLike for BTreeMap<K, V>
where
    K: ToString + Clone + std::fmt::Debug,
    V: ValueLike,
{
    fn into_value(self) -> Value {
        Value::Object(object_from_iter(self))
    }
}

impl<T: ValueCollectionItem> FromIterator<T> for Value {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        T::collect(iter)
    }
}

struct Input<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    end: bool,
    consumed: bool,
    line: i32,
}

impl<'a> Input<'a> {
    fn new(s: &'a str) -> Input<'a> {
        Input {
            chars: s.chars().peekable(),
            end: false,
            consumed: false,
            line: 1,
        }
    }

    fn getc(&mut self) -> Option<char> {
        if self.consumed {
            if let Some(&'\n') = self.chars.peek() {
                self.line += 1;
            }

            let next = self.chars.next();
            if next == None {
                self.end = true;
            }
        }

        if self.end {
            self.consumed = false;
            return None;
        }

        self.consumed = true;
        return self.chars.peek().cloned();
    }

    fn ungetc(&mut self) {
        self.consumed = false;
    }

    fn read_line(&mut self) -> String {
        let mut buf = Vec::new();
        loop {
            match self.getc() {
                None | Some('\n') => {
                    return buf.into_iter().collect::<String>();
                }
                Some(c) => buf.push(c),
            }
        }
    }

    fn skip_ws(&mut self) {
        loop {
            match self.getc() {
                Some(' ') | Some('\t') | Some('\r') | Some('\n') => continue,
                _ => {
                    self.ungetc();
                    break;
                }
            }
        }
    }

    fn expect(&mut self, ex: char) -> bool {
        self.skip_ws();
        match self.getc() {
            Some(c) if c == ex => true,
            _ => {
                self.ungetc();
                false
            }
        }
    }

    fn does_match(&mut self, pattern: &str) -> bool {
        for ex in pattern.chars() {
            match self.getc() {
                Some(c) if c == ex => continue,
                _ => {
                    self.ungetc();
                    return false;
                }
            }
        }
        true
    }
}

trait ParseContext {
    fn set_null(&mut self) -> bool;
    fn set_bool(&mut self, value: bool) -> bool;
    fn set_number(&mut self, value: f64) -> bool;
    fn set_string(&mut self, value: String) -> bool;
    fn parse_array_start(&mut self) -> bool;
    fn parse_array_item<'i>(&mut self, input: &mut Input<'i>, size: usize) -> bool;
    fn parse_array_stop(&mut self, size: usize) -> bool;
    fn parse_object_start(&mut self) -> bool;
    fn parse_object_item<'i>(&mut self, input: &mut Input<'i>, key: String) -> bool;
}

struct DefaultParseContext<'a> {
    out: &'a mut Value,
}

impl<'a> DefaultParseContext<'a> {
    fn new(out: &'a mut Value) -> DefaultParseContext<'a> {
        DefaultParseContext { out }
    }
}

impl<'a> ParseContext for DefaultParseContext<'a> {
    fn set_null(&mut self) -> bool {
        *self.out = Value::Null;
        true
    }

    fn set_bool(&mut self, value: bool) -> bool {
        *self.out = Value::from(value);
        true
    }

    fn set_number(&mut self, value: f64) -> bool {
        *self.out = Value::from(value);
        true
    }

    fn set_string(&mut self, value: String) -> bool {
        *self.out = Value::from(value);
        true
    }

    fn parse_array_start(&mut self) -> bool {
        *self.out = Value::Array(Array::new());
        true
    }

    fn parse_array_item<'i>(&mut self, input: &mut Input<'i>, _size: usize) -> bool {
        let mut value = Value::Null;
        let ok = {
            let mut subcontext = DefaultParseContext::new(&mut value);
            parse_input(&mut subcontext, input)
        };

        if !ok {
            return false;
        }

        let vec = self.out.as_array_mut().unwrap();
        vec.push(value);
        true
    }

    fn parse_array_stop(&mut self, _size: usize) -> bool {
        true
    }

    fn parse_object_start(&mut self) -> bool {
        *self.out = Value::Object(Object::new());
        true
    }

    fn parse_object_item<'i>(&mut self, input: &mut Input<'i>, key: String) -> bool {
        let mut value = Value::Null;
        let ok = {
            let mut subcontext = DefaultParseContext::new(&mut value);
            parse_input(&mut subcontext, input)
        };

        if !ok {
            return false;
        }

        let obj = self.out.as_object_mut().unwrap();
        obj.insert(key, value);
        true
    }
}

fn _read_digits<'a>(input: &mut Input<'a>) -> String {
    let mut num_str = Vec::new();
    loop {
        match input.getc() {
            Some(ch)
                if ('0' <= ch && ch <= '9') || ch == '+' || ch == '-' || ch == 'e' || ch == 'E'
                    || ch == '.' =>
            {
                num_str.push(ch);
            }
            _ => {
                input.ungetc();
                break;
            }
        }
    }
    return num_str.into_iter().collect::<String>();
}

fn _parse_number<'a, C: ParseContext>(ch: char, context: &mut C, input: &mut Input<'a>) -> bool {
    if ('0' <= ch && ch <= '9') || ch == '-' {
        input.ungetc();

        let num_str = _read_digits(input);
        if num_str.is_empty() {
            return false;
        }

        match num_str.parse::<f64>() {
            Ok(value) => {
                context.set_number(value);
                true
            }
            Err(_) => false,
        }
    } else {
        false
    }
}

fn _parse_string<'a>(out: &mut String, input: &mut Input<'a>) -> bool {
    loop {
        match input.getc() {
            None => {
                input.ungetc();
                return false;
            }
            Some(ch) if ch < ' ' => {
                input.ungetc();
                return false;
            }
            Some('"') => {
                return true;
            }
            Some('\\') => {
                match input.getc() {
                    Some('"') => {
                        out.push('\"');
                    }
                    Some('\\') => {
                        out.push('\\');
                    }
                    // Some('/') => {
                    //     out.push('/');
                    // }
                    // Some ('b') => {
                    //     out.push('\b');
                    // }
                    // Some ('f') => {
                    //     out.push('\f');
                    // }
                    Some('n') => {
                        out.push('\n');
                    }
                    Some('r') => {
                        out.push('\r');
                    }
                    Some('t') => {
                        out.push('\t');
                    }
                    Some('u') => {
                        panic!("not implemented");
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Some(ch) => {
                out.push(ch);
            }
        }
    }
}

fn _parse_array<'a, C: ParseContext>(ctx: &mut C, input: &mut Input<'a>) -> bool {
    if !ctx.parse_array_start() {
        return false;
    }

    let mut index = 0;
    if input.expect(']') {
        return ctx.parse_array_stop(index);
    }

    loop {
        if !ctx.parse_array_item(input, index) {
            return false;
        }

        index += 1;
        if !input.expect(',') {
            break;
        }
    }

    if !input.expect(']') {
        return false;
    }

    if !ctx.parse_array_stop(index) {
        return false;
    }

    true
}

fn _parse_object<'a, C: ParseContext>(context: &mut C, input: &mut Input<'a>) -> bool {
    if !context.parse_object_start() {
        return false;
    }

    if input.expect('}') {
        return true;
    }

    loop {
        let mut key = String::new();
        if !input.expect('"') || !_parse_string(&mut key, input) || !input.expect(':') {
            return false;
        }

        if !context.parse_object_item(input, key) {
            return false;
        }

        if !input.expect(',') {
            break;
        }
    }

    if !input.expect('}') {
        return false;
    }

    true
}

fn parse_input<'a, C: ParseContext>(ctx: &mut C, input: &mut Input<'a>) -> bool {
    input.skip_ws();
    match input.getc() {
        Some('n') => {
            if input.does_match("ull") {
                ctx.set_null();
                true
            } else {
                false
            }
        }
        Some('t') => {
            if input.does_match("rue") {
                ctx.set_bool(true);
                true
            } else {
                false
            }
        }
        Some('f') => {
            if input.does_match("alse") {
                ctx.set_bool(false);
                true
            } else {
                false
            }
        }
        Some('[') => _parse_array(ctx, input),
        Some('{') => _parse_object(ctx, input),
        Some(ch) if ('0' <= ch && ch <= '9') || ch == '-' => _parse_number(ch, ctx, input),
        Some('"') => {
            let mut out = String::new();
            if _parse_string(&mut out, input) {
                ctx.set_string(out)
            } else {
                false
            }
        }
        _ => {
            input.ungetc();
            false
        }
    }
}

pub fn parse_string(s: &str) -> Result<Value, Error> {
    let mut out = Value::Null;
    {
        let mut context = DefaultParseContext::new(&mut out);
        let mut input = Input::new(s);
        let ok = parse_input(&mut context, &mut input);
        if !ok {
            input.ungetc();
            let line_number = input.line;
            let near = input.read_line();
            return Err(format!(
                "Syntax error at line {} near: {}",
                line_number, near
            ));
        }
    }
    Ok(out)
}

fn is_first(value: &mut bool) -> bool {
    let old_value = *value;
    *value = false;
    old_value
}

enum JsonSerializationStyle {
    Minimum,
    Pretty {
        indent_level: i32,
        indent_width: i32,
    },
}

type SerializeResult = std::io::Result<()>;

struct JsonSerializer<W> {
    writer: W,
    style: JsonSerializationStyle,
}

#[warn(unused_results)]
impl<W: std::io::Write> JsonSerializer<W> {
    fn write_char(&mut self, c: u8) -> SerializeResult {
        self.writer.write(&[c]).map(|_| ())
    }

    fn write_str(&mut self, s: &str) -> SerializeResult {
        self.writer.write(s.as_bytes()).map(|_| ())
    }

    /// Ends current line and then emits indentation.
    fn end_line(&mut self) -> SerializeResult {
        if let JsonSerializationStyle::Pretty {
            indent_level,
            indent_width,
        } = self.style
        {
            try!(self.write_char(b'\n'));
            for _ in 0..(indent_level * indent_width) {
                try!(self.write_char(b' '));
            }
        }
        return Ok(());
    }

    /// Updates the state to increment indent level by 1 for following lines.
    fn inc_indent(&mut self) {
        if let JsonSerializationStyle::Pretty {
            ref mut indent_level,
            ..
        } = self.style
        {
            *indent_level += 1;
        }
    }

    /// Updates the state to decrease indent level by 1 for following lines.
    fn dec_indent(&mut self) {
        if let JsonSerializationStyle::Pretty {
            ref mut indent_level,
            ..
        } = self.style
        {
            *indent_level -= 1;
        }
    }

    /// Ends final line.
    fn end_final_line(&mut self) -> SerializeResult {
        if let JsonSerializationStyle::Pretty { .. } = self.style {
            try!(self.write_char(b'\n'));
        }
        return Ok(());
    }

    fn write_colon(&mut self) -> SerializeResult {
        match self.style {
            JsonSerializationStyle::Minimum => self.write_char(b':'),
            JsonSerializationStyle::Pretty { .. } => self.write_str(": "),
        }
    }

    fn serialize_number(&mut self, value: f64) -> SerializeResult {
        if value.is_nan() {
            panic!("Can't serialize NaN.")
        } else if value.is_infinite() {
            panic!("Can't serialize infinite number: {}.", value)
        } else {
            self.write_str(&value.to_string())
        }
    }

    fn serialize_string(&mut self, value: &str) -> SerializeResult {
        try!(self.write_char(b'"'));

        for c in value.chars() {
            match c {
                '"' => try!(self.write_str("\\\"")),
                '\n' => try!(self.write_str("\\n")),
                '\r' => try!(self.write_str("\\r")),
                '\t' => try!(self.write_str("\\t")),
                '\\' => try!(self.write_str("\\\\")),
                _ => {
                    fn is_u8(n: i32) -> bool {
                        std::u8::MIN as i32 <= n && n <= std::u8::MAX as i32
                    }
                    if is_u8(c as i32) {
                        try!(self.write_char(c as u8));
                    } else {
                        unimplemented!("\\u+10FFFF")
                    }
                }
            }
        }

        self.write_char(b'"')
    }

    fn serialize_array(&mut self, array: &Array) -> SerializeResult {
        if array.is_empty() {
            self.write_str("[]")
        } else {
            try!(self.write_char(b'['));

            self.inc_indent();
            {
                let mut first = true;
                for item in array {
                    if !is_first(&mut first) {
                        try!(self.write_char(b','));
                    }

                    try!(self.end_line());
                    try!(self.serialize_core(item));
                }
            }
            self.dec_indent();

            try!(self.end_line());
            self.write_char(b']')
        }
    }

    fn serialize_object(&mut self, object: &Object) -> SerializeResult {
        if object.is_empty() {
            self.write_str("{}")
        } else {
            try!(self.write_char(b'{'));

            self.inc_indent();
            {
                let mut first = true;
                for (key, item) in object.iter() {
                    if !is_first(&mut first) {
                        try!(self.write_char(b','));
                    }

                    try!(self.end_line());
                    try!(self.serialize_string(key));
                    try!(self.write_colon());
                    try!(self.serialize_core(item));
                }
            }
            self.dec_indent();

            try!(self.end_line());
            self.write_char(b'}')
        }
    }

    fn serialize_core(&mut self, value: &Value) -> SerializeResult {
        match value {
            &Value::Null => self.write_str("null"),
            &Value::Boolean(true) => self.write_str("true"),
            &Value::Boolean(false) => self.write_str("false"),
            &Value::Number(value) => self.serialize_number(value),
            &Value::String(ref value) => self.serialize_string(value),
            &Value::Array(ref array) => self.serialize_array(array),
            &Value::Object(ref object) => self.serialize_object(object),
        }
    }

    fn serialize(&mut self, value: &Value) -> SerializeResult {
        try!(self.serialize_core(value));
        self.end_final_line()
    }
}

/// Serializes a value with compress style.
/// Writes the utf-8 encoded string to the specified writer.
pub fn serialize<W: std::io::Write>(value: &Value, writer: &mut W) -> SerializeResult {
    JsonSerializer {
        writer,
        style: JsonSerializationStyle::Minimum,
    }.serialize(value)
}

/// Serializes a value with space-indented style.
/// Writes the utf-8 encoded string to the specified writer.
pub fn pretty_print<W: std::io::Write>(
    value: &Value,
    indent_width: i32,
    writer: &mut W,
) -> SerializeResult {
    JsonSerializer {
        writer,
        style: JsonSerializationStyle::Pretty {
            indent_level: 0,
            indent_width,
        },
    }.serialize(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn test_object_comparison() {
        let obj1 = object_from_iter(vec![("a", 1)]);
        let obj2 = object_from_iter(vec![("a", Value::Null)]);

        assert_eq!(obj1, obj1);
        assert_eq!(obj1.partial_cmp(&obj2), Some(Ordering::Greater));
    }

    #[test]
    fn test_value_from() {
        assert_eq!(Value::from(true), Value::Boolean(true));
        assert_eq!(Value::from(42), Value::Number(42.0));
        assert_eq!(Value::from(3.14), Value::Number(3.14));
    }

    #[test]
    fn test_value_is_empty() {
        assert_eq!(Value::from("").is_empty(), false);
    }

    #[test]
    fn test_value_has() {
        assert_eq!(Value::Null.has(0), false);
        assert_eq!(Value::Null.has("unknown"), false);

        assert_eq!(Value::from(vec!["a", "b", "c"]).has(2), true);
    }

    #[test]
    fn test_input_getc() {
        let source = "12";
        let mut input = Input::new(source);
        assert_eq!(input.getc(), Some('1'));
        assert_eq!(input.getc(), Some('2'));
        assert_eq!(input.getc(), None);
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_string("1"), Ok(Value::Number(1.0)));
        assert_eq!(parse_string("-3.14"), Ok(Value::Number(-3.14)));
        assert_eq!(parse_string("1e-9"), Ok(Value::Number(1e-9)));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_string(r#""hello""#),
            Ok(Value::String("hello".to_string()))
        );

        // Empty.
        assert_eq!(parse_string(r#""""#), Ok(Value::String("".to_string())));

        // Escapse sequences.
        assert_eq!(
            parse_string(r#" "Hello,\r\n\tworld!" "#),
            Ok(Value::String("Hello,\r\n\tworld!".to_string()))
        );

        // Unknown escapse sequence.
        assert!(parse_string(r#" "\q" "#).is_err());

        // Not closed.
        assert!(parse_string(r#" "hello-- "#).is_err());
    }

    #[test]
    fn test_parse_string_as_array() {
        assert_eq!(parse_string("[]"), Ok(Value::array()));
        assert_eq!(
            parse_string("[true, false, true]"),
            Ok(Value::from(vec![true, false, true]))
        );
    }

    #[test]
    fn test_parse_object_empty() {
        assert_eq!(parse_string("{}"), Ok(Value::object()));
    }

    #[test]
    fn test_parse_object_simple() {
        let json = r#"{
            "user_id": 42,
            "name": "John Doe"
        }"#;
        let expected = Value::from(vec![
            ("user_id", Value::from(42)),
            ("name", Value::from("John Doe")),
        ]);
        assert_eq!(parse_string(json), Ok(expected));
    }

    #[test]
    fn test_parse_object_nested() {
        let json = r#"{
            "array": [1, 2, 3],
            "object": { "foo": true }
        }"#;
        let expected = Value::from(vec![
            ("array", Value::from(vec![1, 2, 3])),
            ("object", Value::from(vec![("foo", true)])),
        ]);
        assert_eq!(parse_string(json), Ok(expected));
    }

    #[test]
    fn test_parse_object_rejects_dangling_comma() {
        assert!(parse_string(r#"{ "foo": true, }"#).is_err());
    }

    #[test]
    fn test_parse_bool_ok() {
        assert_eq!(parse_string("true"), Ok(Value::Boolean(true)));
        assert_eq!(parse_string("false"), Ok(Value::Boolean(false)));

        // Ignore leading whitespaces.
        assert_eq!(parse_string("\t\n\r  true"), Ok(Value::Boolean(true)));
    }

    #[test]
    fn test_parse_bool_err() {
        // Typo.
        assert!(parse_string("ture").is_err());

        // Case sensitive.
        assert!(parse_string("TRUE").is_err());
    }

    #[test]
    #[should_panic]
    fn test_serialize_number_nan() {
        Value::Number(std::f64::NAN).serialize();
    }

    #[test]
    #[should_panic]
    fn test_serialize_number_infinity() {
        Value::Number(std::f64::INFINITY).serialize();
    }

    #[test]
    fn test_serialize_string_simple() {
        assert_eq!(Value::from("hello").serialize(), r#""hello""#);
    }

    #[test]
    fn test_serialize_string_escaped() {
        let source = "\"\"\"\n\tHello!\n\"\"\"";
        let expected = r#""\"\"\"\n\tHello!\n\"\"\"""#;
        assert_eq!(Value::from(source).serialize(), expected);
    }

    #[test]
    #[ignore]
    fn test_serialize_string_unicode() {
        assert_eq!(Value::from("你好").serialize(), r#""\u4f60\u597d""#);
    }

    #[test]
    fn test_pretty_print_array() {
        let expected = r#"[
  42,
  "hello",
  [
    "nested",
    "items"
  ]
]
"#;
        let actual = Value::from(vec![
            Value::from(42),
            Value::from("hello"),
            Value::from(vec!["nested", "items"]),
        ]);
        assert_eq!(actual.pretty_print(), expected);
    }
}

#[cfg(test)]
mod ported_tests {
    use super::*;
    use std::*;

    #[test]
    fn test_constructor() {
        let table = vec![
            (Value::from(true), "true"),
            (Value::from(false), "false"),
            (Value::from(42.0), "42"),
            (Value::from("hello"), "\"hello\""),
        ];
        for (value, json) in table {
            let actual = value.serialize();
            assert_eq!(actual, json);
        }
    }

    #[test]
    fn test_double_reserialization() {
        /// Serialize and deserialize a number.
        fn f(r: f64) -> f64 {
            let json = Value::from(r).serialize();
            let value = parse_string(&json).unwrap();
            value.as_number().cloned().unwrap()
        }

        for n in 1..53 {
            let x = (1_i64 << n) as f64;
            assert_eq!(f(x), x);
        }

        for n in 53..1024 {
            let x = 1_f64.powf(n as f64);
            let y = f(x);
            assert!((x - y).abs() / y <= 1e-8);
        }
    }

    #[test]
    #[cfg(not_impl)]
    fn test_correct_output() {
        fn test<T>(source: String, expected: T) {
            let value = parse_string(source).unwrap();
            assert!(value.is_of<T>())
            // value: has type T
            // parsed completely
        }

        test("false", false);
        test("true", true);
        test("90.5", 90.5_f64);
        test("1.7976931348623157e+308", 1.7976931348623157e+308_f64);
        test("\"hello\"", "hello".to_string());
        test("\"\\\"\\\\\\/\\n\\r\\t\"", "\"\\/\n\r\t".to_string());
    }

    #[test]
    fn test_value_is_empty() {
        assert!(parse_string("[]").expect("Parse success").is_empty());
        assert!(parse_string("{}").expect("Parse success").is_empty());
    }

    #[test]
    fn test_value_array() {
        let a = parse_string("[1,true,\"hello\"]").expect("Should parse an array");

        assert_eq!(a.as_array().unwrap().len(), 3);

        assert_eq!(a.has(0), true, "First element should exist.");
        assert_eq!(a.get(0).unwrap().as_number().unwrap(), &1.0);

        assert_eq!(a.has(1), true, "Second element should exist.");
        assert_eq!(a.get(1).unwrap().as_bool().unwrap(), &true);

        assert_eq!(a.has(2), true, "Third element should exist.");
        assert_eq!(a.get(2).unwrap().as_string().unwrap(), "hello");

        assert!(!a.has(3));
    }

    #[test]
    fn test_value_object() {
        let v = parse_string(r#"{ "a": true }"#).expect("Should parse an object");

        assert_eq!(v.as_object().unwrap().len(), 1);
        assert_eq!(v.has("a"), true, "Should has a as key.");
        assert_eq!(v.get("a"), Some(&Value::from(true)));

        assert!(!v.has("z"));
    }

    #[test]
    fn test_value_object_modification() {
        let mut v = Value::object();

        v.insert("foo", Value::from("bar"));

        v.insert("hoge", Value::array());
        v.get_mut("hoge").unwrap().push(Value::from(42.0));

        v.insert("baz", Value::object());
        {
            let v2 = v.get_mut("baz").unwrap();
            v2.insert("piyo", Value::from(3.14));
        }

        let json = v.serialize();
        assert_eq!(json, r#"{"baz":{"piyo":3.14},"foo":"bar","hoge":[42]}"#);
    }

    #[test]
    fn test_error_message() {
        fn test(source: &str, line: i32, near: &str) {
            let actual = parse_string(source).expect_err("Expected an error.");
            let expected = format!("Syntax error at line {} near: {}", line, near);
            assert_eq!(actual, expected);
        }

        test("falsoa", 1, "oa");
        test("{]", 1, "]");
        test("\n\tbell", 2, "bell");
        test("\"abc\nd\"", 1, "");
    }

    #[test]
    fn test_deep_comparison_equal() {
        let l = parse_string(r#"{ "b": true, "a": [1, 2, "three"], "d": 2 }"#);
        let r = parse_string(r#"{ "d": 2.0, "b": true, "a": [1, 2, "three"] }"#);
        assert_eq!(l, r);
    }

    #[test]
    #[should_panic]
    fn test_deep_comparison_not_equal_array() {
        let l = parse_string(r#"{ "b": true, "a": [1, 2, "three"], "d": 2 }"#);
        let r = parse_string(r#"{ "b": true, "a": [1,    "three"], "d": 2 }"#);
        assert_eq!(l, r);
    }

    #[test]
    #[should_panic]
    fn test_deep_comparison_not_equal_boolean() {
        let l = parse_string(r#"{ "b": true, "a": [1, 2, "three"], "d": 2 }"#);
        let r = parse_string(r#"{ "b":false, "a": [1, 2, "three"], "d": 2 }"#);
        assert_eq!(l, r);
    }

    #[test]
    #[cfg(not_impl)]
    fn test_erase() {
        let obj = parse_string(r#"{ "b": true, "a": [1, 2, "three"], "d": 2 }"#).unwrap();
        obj.erase("b");
        let expected = parse_string(r#"{ a": [1, 2, "three"], "d": 2 }"#).unwrap();
        assert_eq!(obj, expected);
    }

    #[test]
    fn test_serialize_integer() {
        assert_eq!(Value::from(2.0).serialize(), "2");
    }

    fn serialization_sample() -> Value {
        parse_string(
            r#"{
            "a": 1,
            "b": [2, { "b1": "abc" }],
            "c": {},
            "d": []
        }"#,
        ).unwrap()
    }

    #[test]
    fn test_serialize_object_minimum() {
        let actual = serialization_sample().serialize();
        assert_eq!(actual, r#"{"a":1,"b":[2,{"b1":"abc"}],"c":{},"d":[]}"#);
    }

    #[test]
    fn test_pretty_print_object() {
        let actual = serialization_sample().pretty_print();
        let expected = r#"{
  "a": 1,
  "b": [
    2,
    {
      "b1": "abc"
    }
  ],
  "c": {},
  "d": []
}
"#;
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_reject_nan() {
        Value::from(f64::NAN);
    }

    #[test]
    #[should_panic]
    fn test_reject_infinity() {
        Value::from(f64::INFINITY);
    }

    #[test]
    fn test_cast() {
        assert_eq!(Value::from(3.14).as_bool(), None);
    }

    #[test]
    #[cfg(not_impl)]
    fn test_simple_api() {
        let v = parse_string(r#"[ 1, "abc" ]"#).unwrap();
        let a = v.as_array().unwrap();
        assert_eq!(a.len(), 2);
        assert_eq!(a[0].as_number().unwrap(), 1);
        assert_eq!(a[1].as_string().unwrap(), "abc");
    }
}
