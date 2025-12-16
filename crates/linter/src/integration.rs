use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Ord, Copy, Clone, Hash, PartialOrd, Deserialize, Serialize, JsonSchema)]
#[schemars(with = "Vec<Integration>")]
pub struct IntegrationSet(u32);

#[derive(Debug, PartialEq, Eq, Ord, Copy, Clone, Hash, PartialOrd, Serialize, JsonSchema)]
#[schemars(with = "String")]
#[repr(u8)]
pub enum Integration {
    // Libraries
    Psl,
    Guzzle,
    Monolog,
    Carbon,
    Amphp,
    ReactPHP,
    // Frameworks
    Symfony,
    Laravel,
    Tempest,
    Neutomic,
    Spiral,
    CakePHP,
    Yii,
    Laminas,
    // ORMs
    Cycle,
    Doctrine,
    // CMS
    WordPress,
    Drupal,
    Magento,
    // Testing Frameworks
    PHPUnit,
    Pest,
    Behat,
    Codeception,
    PHPSpec,
}

impl IntegrationSet {
    #[must_use]
    pub const fn all() -> Self {
        Self(u32::MAX)
    }

    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    #[must_use]
    pub const fn only(integration: Integration) -> Self {
        let mut s = Self::empty();
        s.0 |= 1 << (integration as u32);
        s
    }

    #[inline]
    pub fn insert(&mut self, integration: Integration) {
        self.0 |= 1 << (integration as u32);
    }

    #[inline]
    #[must_use]
    pub const fn contains(&self, lib: Integration) -> bool {
        (self.0 & (1 << (lib as u32))) != 0
    }

    /// Checks if this set has any integrations in common with another set.
    #[inline]
    #[must_use]
    pub const fn intersects(&self, other: IntegrationSet) -> bool {
        (self.0 & other.0) != 0
    }

    /// Creates a new set containing all integrations from both sets.
    #[inline]
    #[must_use]
    pub const fn union(&self, other: IntegrationSet) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    #[must_use]
    pub const fn is_superset_of(&self, other: IntegrationSet) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    #[must_use]
    pub const fn from_slice(xs: &[Integration]) -> Self {
        let mut s = IntegrationSet::empty();
        let mut i = 0;
        while i < xs.len() {
            s.0 |= 1 << (xs[i] as u32);
            i += 1;
        }

        s
    }
}

impl std::str::FromStr for Integration {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.replace('_', "-").to_lowercase().as_str() {
            "psl" | "php-standard-library" => Ok(Integration::Psl),
            "guzzle" => Ok(Integration::Guzzle),
            "monolog" => Ok(Integration::Monolog),
            "carbon" => Ok(Integration::Carbon),
            "amphp" | "amp" => Ok(Integration::Amphp),
            "reactphp" | "react-php" | "react" => Ok(Integration::ReactPHP),
            "symfony" => Ok(Integration::Symfony),
            "laravel" => Ok(Integration::Laravel),
            "tempest" => Ok(Integration::Tempest),
            "neutomic" => Ok(Integration::Neutomic),
            "spiral" => Ok(Integration::Spiral),
            "cakephp" | "cake-php" => Ok(Integration::CakePHP),
            "yii" => Ok(Integration::Yii),
            "laminas" => Ok(Integration::Laminas),
            "cycle" => Ok(Integration::Cycle),
            "doctrine" => Ok(Integration::Doctrine),
            "wordpress" | "wp" => Ok(Integration::WordPress),
            "drupal" => Ok(Integration::Drupal),
            "magento" => Ok(Integration::Magento),
            "phpunit" | "php-unit" => Ok(Integration::PHPUnit),
            "pest" => Ok(Integration::Pest),
            "behat" => Ok(Integration::Behat),
            "codeception" | "code-ception" => Ok(Integration::Codeception),
            "phpspec" | "php-spec" => Ok(Integration::PHPSpec),
            _ => Err("unknown integration"),
        }
    }
}

impl<'de> Deserialize<'de> for Integration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Visitor;

        struct IntegrationVisitor;

        impl Visitor<'_> for IntegrationVisitor {
            type Value = Integration;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "a string representing an integration, such as 'psl', 'symfony', 'laravel', or 'phpunit'",
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                std::str::FromStr::from_str(v).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(IntegrationVisitor)
    }
}

impl std::fmt::Display for Integration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Integration::Psl => write!(f, "Psl"),
            Integration::Guzzle => write!(f, "Guzzle"),
            Integration::Monolog => write!(f, "Monolog"),
            Integration::Carbon => write!(f, "Carbon"),
            Integration::Amphp => write!(f, "Amphp"),
            Integration::ReactPHP => write!(f, "ReactPHP"),
            Integration::Symfony => write!(f, "Symfony"),
            Integration::Laravel => write!(f, "Laravel"),
            Integration::PHPUnit => write!(f, "PHPUnit"),
            Integration::Tempest => write!(f, "Tempest"),
            Integration::Neutomic => write!(f, "Neutomic"),
            Integration::Spiral => write!(f, "Spiral"),
            Integration::CakePHP => write!(f, "CakePHP"),
            Integration::Yii => write!(f, "Yii"),
            Integration::Laminas => write!(f, "Laminas"),
            Integration::Cycle => write!(f, "Cycle"),
            Integration::Doctrine => write!(f, "Doctrine"),
            Integration::WordPress => write!(f, "WordPress"),
            Integration::Drupal => write!(f, "Drupal"),
            Integration::Magento => write!(f, "Magento"),
            Integration::Pest => write!(f, "Pest"),
            Integration::Behat => write!(f, "Behat"),
            Integration::Codeception => write!(f, "Codeception"),
            Integration::PHPSpec => write!(f, "PHPSpec"),
        }
    }
}
