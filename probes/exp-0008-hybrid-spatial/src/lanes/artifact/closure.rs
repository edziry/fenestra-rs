use std::collections::BTreeSet;

use super::sha256::sha256_hex_v2;

const LOCK: &str = include_str!("../../../../../Cargo.lock");

#[derive(Clone)]
struct Package<'a> {
    name: &'a str,
    version: &'a str,
    dependencies: Vec<Dependency<'a>>,
}

#[derive(Clone, Copy)]
struct Dependency<'a> {
    name: &'a str,
    version: Option<&'a str>,
}

pub(super) fn closure_sha256_v2(roots: &[(&str, &str)]) -> Result<String, &'static str> {
    let packages = packages()?;
    let mut pending = roots.to_vec();
    let mut closure = BTreeSet::new();
    while let Some((name, version)) = pending.pop() {
        if !closure.insert((name, version)) {
            continue;
        }
        let package = find_exact(&packages, name, version)?;
        for dependency in &package.dependencies {
            let resolved = resolve(&packages, *dependency)?;
            pending.push((resolved.name, resolved.version));
        }
    }
    let mut canonical = String::new();
    for (name, version) in closure {
        canonical.push_str(name);
        canonical.push('@');
        canonical.push_str(version);
        canonical.push('\n');
    }
    Ok(sha256_hex_v2(canonical.as_bytes()))
}

fn packages() -> Result<Vec<Package<'static>>, &'static str> {
    LOCK.split("[[package]]")
        .skip(1)
        .map(parse_package)
        .collect()
}

fn parse_package(section: &'static str) -> Result<Package<'static>, &'static str> {
    let mut name = None;
    let mut version = None;
    let mut dependencies = Vec::new();
    let mut inside_dependencies = false;
    for line in section.lines().map(str::trim) {
        if line == "dependencies = [" {
            inside_dependencies = true;
            continue;
        }
        if inside_dependencies {
            if line == "]" {
                inside_dependencies = false;
            } else if !line.is_empty() {
                dependencies.push(parse_dependency(line)?);
            }
            continue;
        }
        if name.is_none() {
            name = quoted_value(line, "name = ");
        }
        if version.is_none() {
            version = quoted_value(line, "version = ");
        }
    }
    Ok(Package {
        name: name.ok_or("lock package has no name")?,
        version: version.ok_or("lock package has no version")?,
        dependencies,
    })
}

fn parse_dependency(line: &'static str) -> Result<Dependency<'static>, &'static str> {
    let value = line
        .strip_suffix(',')
        .unwrap_or(line)
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .ok_or("invalid lock dependency")?;
    let mut words = value.split_whitespace();
    let name = words.next().ok_or("empty lock dependency")?;
    let version = words
        .next()
        .filter(|word| word.as_bytes().first().is_some_and(u8::is_ascii_digit));
    Ok(Dependency { name, version })
}

fn quoted_value<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    line.strip_prefix(prefix)?
        .strip_prefix('"')?
        .strip_suffix('"')
}

fn find_exact<'a>(
    packages: &'a [Package<'a>],
    name: &str,
    version: &str,
) -> Result<&'a Package<'a>, &'static str> {
    let mut matches = packages
        .iter()
        .filter(|package| package.name == name && package.version == version);
    let package = matches
        .next()
        .ok_or("registered package missing from lock")?;
    if matches.next().is_some() {
        return Err("duplicate registered package in lock");
    }
    Ok(package)
}

fn resolve<'a>(
    packages: &'a [Package<'a>],
    dependency: Dependency<'_>,
) -> Result<&'a Package<'a>, &'static str> {
    let mut matches = packages.iter().filter(|package| {
        package.name == dependency.name
            && dependency
                .version
                .is_none_or(|version| package.version == version)
    });
    let package = matches.next().ok_or("lock dependency is unresolved")?;
    if matches.next().is_some() {
        return Err("lock dependency is ambiguous");
    }
    Ok(package)
}
