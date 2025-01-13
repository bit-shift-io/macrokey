#[derive(Clone, Copy, Debug)]
pub enum KeyEventType {
    RELEASED,
    PRESSED,
    REPEAT,
    UNKNOWN(i32),
}

impl KeyEventType {
    fn from_value(value: i32) -> Self {
        match value {
            0 => KeyEventType::RELEASED,
            1 => KeyEventType::PRESSED,
            2 => KeyEventType::REPEAT,
            _ => KeyEventType::UNKNOWN(value),
        }
    }

    fn value(&self) -> i32 {
        match self {
            Self::RELEASED => 0,
            Self::PRESSED => 1,
            Self::REPEAT => 2,
            Self::UNKNOWN(n) => *n,
        }
    }
}

impl PartialEq<i32> for KeyEventType {
    fn eq(&self, other: &i32) -> bool {
        self.value() == *other
    }
}

impl PartialEq<KeyEventType> for i32 {
    fn eq(&self, other: &KeyEventType) -> bool {
        *self == other.value()
    }
}

impl Into<i32> for KeyEventType {
    fn into(self) -> i32 {
        self.value()
    }
}