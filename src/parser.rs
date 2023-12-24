#[derive(Debug, Copy, Clone)]
pub struct Bite<'a> {
    inner: &'a str,
}

impl<'a> From<&'a str> for Bite<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> Bite<'a> {
    pub fn new(inner: &'a str) -> Self {
        Self { inner }
    }
    pub fn chomp<M: ChompMatcher<'a>>(self, chomp: Chomp<M>) -> Self {
        let (_, bite) = chomp.consume(self);
        bite
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    pub fn as_str(&self) -> &str {
        self.inner
    }
    pub fn can_nibble<M: ChompMatcher<'a>>(&self, chomp: Chomp<M>) -> bool {
        chomp.is_match(*self)
    }
    pub fn nibble<M: ChompMatcher<'a>>(&mut self, chomp: Chomp<M>) -> Option<&'a str> {
        let (matched, bite) = chomp.consume(*self);
        *self = bite;
        matched.filter(|x| !x.is_empty())
    }
    pub fn nibble_entire<M: ChompMatcher<'a>, const N: usize>(
        &mut self,
        chomp: [Chomp<M>; N],
    ) -> Option<[&'a str; N]> {
        let mut matches = [""; N];
        let mut next = *self;
        for (mut chomp, nibble_match) in chomp.into_iter().zip(matches.iter_mut()) {
            (*nibble_match, next) = chomp.matcher.consume(next).filter(|(x, _)| !x.is_empty())?;
        }
        *self = next;
        Some(matches)
    }
    pub fn swallow_char(&mut self) -> Option<char> {
        let c = self.inner.chars().next()?;
        let (_, rest) = self.inner.split_at(c.len_utf8());
        self.inner = rest;
        Some(c)
    }
}

pub struct Chomp<M> {
    matcher: M,
}

impl<'a> Chomp<()> {
    pub fn whitespace() -> Chomp<fn(&'a str) -> Option<usize>> {
        Chomp {
            matcher: matchers::is_whitespace,
        }
    }
    pub fn alphabetic() -> Chomp<fn(&'a str) -> Option<usize>> {
        Chomp {
            matcher: matchers::is_alphabetic,
        }
    }
    pub fn alphanumeric() -> Chomp<fn(&'a str) -> Option<usize>> {
        Chomp {
            matcher: matchers::is_alphanumeric,
        }
    }
    pub fn alphanumeric_extended() -> Chomp<fn(&'a str) -> Option<usize>> {
        Chomp {
            matcher: matchers::is_alphanumeric_extended,
        }
    }
    pub fn numeric() -> Chomp<fn(&'a str) -> Option<usize>> {
        Chomp {
            matcher: matchers::is_numeric,
        }
    }
    pub fn any_number() -> Chomp<impl FnMut(&'a str) -> Option<usize>> {
        let mut seen_dp = false;
        Chomp {
            matcher: move |x: &str| {
                if x.split_once(|x: char| !x.is_ascii_digit() && !['.', '-', '−'].contains(&x))
                    .map_or(false, |(x, _)| {
                        x.len() <= 2 && !x.starts_with(|c| char::is_ascii_digit(&c))
                    })
                {
                    return None;
                }
                matchers::matches(
                    |z| match z {
                        (0, '-' | '−') => true,
                        (_, '.') if !seen_dp => {
                            seen_dp = true;
                            true
                        }
                        (_, '0'..='9') => true,
                        _ => false,
                    },
                    x,
                )
            },
        }
    }
    pub fn literal(pattern: &'a str) -> Chomp<impl FnMut(&'a str) -> Option<usize>> {
        Chomp {
            matcher: move |x: &str| {
                x.starts_with(pattern)
                    .then(|| {
                        !x[pattern.len()..].starts_with(|c: char| c.is_alphabetic() || c == '_')
                    })
                    .and_then(|x| x.then_some(pattern.len()))
            },
        }
    }
    pub fn literal_substring(pattern: &'a str) -> Chomp<impl FnMut(&'a str) -> Option<usize>> {
        Chomp {
            matcher: move |x: &str| x.starts_with(pattern).then(|| pattern.len()),
        }
    }
    pub fn char(c: char) -> Chomp<impl Fn(&'a str) -> Option<usize>> {
        Chomp {
            matcher: move |x: &str| matchers::char_matches(move |(_, x)| *x == c, x),
        }
    }
    pub fn char_any<const N: usize>(c: [char; N]) -> Chomp<impl Fn(&'a str) -> Option<usize>> {
        Chomp {
            matcher: move |x: &str| {
                matchers::char_matches(move |(_, x)| c.into_iter().any(|c| *x == c), x)
            },
        }
    }
}

impl<'a, M: FnOnce(&'a str) -> Option<usize>> Chomp<M> {}

impl<'a, M: ChompMatcher<'a>> Chomp<M> {
    pub fn new(matcher: M) -> Self {
        Self { matcher }
    }
    pub fn is_match(mut self, bite: Bite<'a>) -> bool {
        let consume = self.matcher.consume(bite);
        consume.map_or(false, |(matched, _)| !matched.is_empty())
    }
    pub fn consume(mut self, bite: Bite<'a>) -> (Option<&'a str>, Bite<'a>) {
        let consume = self.matcher.consume(bite);
        let map = consume.map(|(matched, bite)| (Some(matched), bite));
        map.unwrap_or((None, bite))
    }
    pub fn consume_until(mut self, bite: Bite<'a>) -> (Option<&'a str>, Bite<'a>) {
        let consume = self.matcher.consume(bite);
        let map = consume.map(|(matched, bite)| (Some(matched), bite));
        map.unwrap_or((None, bite))
    }
    pub fn or<T>(self, other: Chomp<T>) -> Chomp<matchers::combine::Or<M, T>>
    where
        T: ChompMatcher<'a>,
    {
        Chomp {
            matcher: matchers::combine::Or::new(self.matcher, other.matcher),
        }
    }
}

mod matchers {
    pub fn is_whitespace(x: &str) -> Option<usize> {
        matches(|(_, c)| c.is_whitespace(), x)
    }
    pub fn is_alphabetic(x: &str) -> Option<usize> {
        matches(|(_, c)| c.is_alphabetic(), x)
    }
    pub fn is_alphanumeric(x: &str) -> Option<usize> {
        matches(|(_, c)| c.is_alphanumeric(), x)
    }
    pub fn is_alphanumeric_extended(x: &str) -> Option<usize> {
        matches(|(_, c)| c.is_alphanumeric() || matches!(c, '_'), x)
    }
    pub fn is_numeric(x: &str) -> Option<usize> {
        matches(|(_, c)| c.is_numeric(), x)
    }
    pub fn matches(f: impl FnMut(&(usize, char)) -> bool, x: &str) -> Option<usize> {
        x.char_indices()
            .chain(std::iter::once((x.len(), '\x00')))
            .skip_while(f)
            .next()
            .map(|(i, _)| i)
    }
    pub fn char_matches(f: impl FnOnce(&(usize, char)) -> bool, x: &str) -> Option<usize> {
        match x.chars().next() {
            Some(x) if f(&(0, x)) => Some(x.len_utf8()),
            _ => None,
        }
    }

    pub mod combine {
        use crate::parser::{Bite, ChompMatcher};

        pub struct Or<T1, T2>(T1, T2);

        impl<'a, T1: ChompMatcher<'a>, T2: ChompMatcher<'a>> Or<T1, T2> {
            pub fn new(first: T1, second: T2) -> Self {
                Self(first, second)
            }
        }

        impl<'a, T1, T2> ChompMatcher<'a> for Or<T1, T2>
        where
            T1: ChompMatcher<'a>,
            T2: ChompMatcher<'a>,
        {
            fn consume(&mut self, bite: Bite<'a>) -> Option<(&'a str, Bite<'a>)> {
                self.0.consume(bite).or_else(move || self.1.consume(bite))
            }
        }
    }
}

pub trait ChompMatcher<'a> {
    fn consume(&mut self, bite: Bite<'a>) -> Option<(&'a str, Bite<'a>)>;
    fn consume_char(&mut self, bite: Bite<'a>) -> Option<(char, Bite<'a>)> {
        self.consume(bite)
            .filter(|(matched, _)| !matched.is_empty())?;
        let mut bite = bite.clone();
        let c = bite.swallow_char()?;
        Some((c, bite))
    }
}

impl<'a, T> ChompMatcher<'a> for T
where
    T: FnMut(&'a str) -> Option<usize>,
{
    fn consume(&mut self, bite: Bite<'a>) -> Option<(&'a str, Bite<'a>)> {
        let mid = self(bite.inner)?;
        let (matched, remaining) = bite.inner.split_at(mid);
        Some((matched, Bite::new(remaining)))
    }
}
