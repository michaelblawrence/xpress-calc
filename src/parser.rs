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
        for (chomp, nibble_match) in chomp.into_iter().zip(matches.iter_mut()) {
            (*nibble_match, next) = chomp.matcher.consume(next).filter(|(x, _)| !x.is_empty())?;
        }
        *self = next;
        Some(matches)
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
    pub fn numeric() -> Chomp<fn(&'a str) -> Option<usize>> {
        Chomp {
            matcher: matchers::is_numeric,
        }
    }
    pub fn any_number() -> Chomp<impl FnOnce(&'a str) -> Option<usize>> {
        let mut seen_dp = false;
        Chomp {
            matcher: move |x: &str| {
                matchers::matches(
                    |x| match x {
                        (0, '+' | '-') => true,
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
    pub fn literal(pattern: &'a str) -> Chomp<impl FnOnce(&'a str) -> Option<usize>> {
        let mut char_indices = pattern.char_indices();
        Chomp {
            matcher: move |x: &str| {
                matchers::matches(|x| char_indices.next().map_or(false, |c| x == &c), x)
            },
        }
    }
    pub fn char(c: char) -> Chomp<impl FnOnce(&'a str) -> Option<usize>> {
        Chomp {
            matcher: move |x: &str| matchers::char_matches(move |(_, x)| *x == c, x),
        }
    }
    pub fn char_any(c: &'a [char]) -> Chomp<impl FnOnce(&'a str) -> Option<usize>> {
        Chomp {
            matcher: move |x: &str| matchers::char_matches(move |(_, x)| c.contains(x), x),
        }
    }
}

impl<'a, M: FnOnce(&'a str) -> Option<usize>> Chomp<M> {
    pub fn or(
        self,
        other: Chomp<impl FnOnce(&'a str) -> Option<usize>>,
    ) -> Chomp<impl FnOnce(&'a str) -> Option<usize>> {
        Chomp {
            matcher: move |x: &'a str| ((self.matcher)(x).or_else(move || (other.matcher)(x))),
        }
    }
}

impl<'a, M: ChompMatcher<'a>> Chomp<M> {
    pub fn new(matcher: M) -> Self {
        Self { matcher }
    }
    pub fn consume(self, bite: Bite<'a>) -> (Option<&'a str>, Bite<'a>) {
        let consume = self.matcher.consume(bite);
        let map = consume.map(|(matched, bite)| (Some(matched), bite));
        map.unwrap_or((None, bite))
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
}

pub trait ChompMatcher<'a> {
    fn consume(self, bite: Bite<'a>) -> Option<(&'a str, Bite<'a>)>;
}

impl<'a, T> ChompMatcher<'a> for T
where
    T: FnOnce(&'a str) -> Option<usize>,
{
    fn consume(self, bite: Bite<'a>) -> Option<(&'a str, Bite<'a>)> {
        let mid = self(bite.inner)?;
        let (matched, remaining) = bite.inner.split_at(mid);
        Some((matched, Bite::new(remaining)))
    }
}
