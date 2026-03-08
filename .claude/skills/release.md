---
name: release
description: Create a new Ferrite release — bumps version strings across all files, commits, tags, builds DMG, and publishes a GitHub release with auto-generated release notes. Use this skill whenever the user says "release", "new version", "ship it", "cut a release", "publish a build", "make a release", or anything about creating/publishing a new version of Ferrite.
---

# Release Ferrite

End-to-end release workflow: version bump → commit → tag → build DMG → GitHub release.

## Prerequisites

Before starting, verify:
- Working tree is clean (`git status` shows no uncommitted changes)
- On the `main` branch
- `gh` CLI is authenticated (`gh auth status`)
- Rust toolchain and Xcode CLI tools are available

If any prerequisite fails, stop and tell the user what needs fixing.

## Step 1: Determine the new version

Ask the user what version to release. Show the current version (from `src/swift/Ferrite/Info.plist`, the `CFBundleShortVersionString` value) and the last git tag. Suggest the next patch/minor/major bump as options.

Format: `MAJOR.MINOR.PATCH` (no `v` prefix in files, `v` prefix on git tags).

## Step 2: Bump version strings

Update the version in these 3 files (and only these 3):

| File | Field | Example |
|------|-------|---------|
| `src/swift/Ferrite/Info.plist` | `<string>` after `CFBundleShortVersionString` | `0.2.0` |
| `src/rust/ferrite-pe/Cargo.toml` | `version = "..."` | `0.2.0` |
| `src/rust/ferrite-ffi/Cargo.toml` | `version = "..."` | `0.2.0` |

Use the Edit tool for each file. Do NOT touch `CFBundleVersion` (build number) — leave it as-is.

## Step 3: Generate release notes

Run: `git log $(git describe --tags --abbrev=0)..HEAD --oneline --no-decorate`

This gives all commits since the last tag. Format them into release notes:

```
## What's New
- <human-readable summary of each meaningful commit>

## Full Changelog
<previous-tag>...v<new-version>
```

Group related commits. Skip merge commits and trivial ones (formatting, typos). Present the draft to the user for review before continuing.

## Step 4: Commit and tag

Stage and commit the 3 modified files:
```bash
git add src/swift/Ferrite/Info.plist src/rust/ferrite-pe/Cargo.toml src/rust/ferrite-ffi/Cargo.toml
git commit -m "release: v<VERSION>"
```

Then create an annotated tag:
```bash
git tag -a "v<VERSION>" -m "v<VERSION>"
```

## Step 5: Build the DMG

Run:
```bash
make dmg
```

This runs the full pipeline: Rust build → Swift bindings → Xcode archive → DMG creation. The DMG will be at `build/Ferrite-v<VERSION>.dmg`.

If the build fails, stop and help debug. Do NOT proceed to publishing with a broken build.

Verify the DMG was created:
```bash
ls -lh build/Ferrite-v<VERSION>.dmg
```

## Step 6: Push and publish

Push the commit and tag:
```bash
git push origin main --follow-tags
```

Create the GitHub release with the DMG attached:
```bash
gh release create "v<VERSION>" \
  "build/Ferrite-v<VERSION>.dmg" \
  --title "Ferrite v<VERSION>" \
  --notes "<release-notes>"
```

## Step 7: Confirm

After publishing, show the user:
- The release URL (from `gh release view` output)
- A summary: version, DMG size, number of commits included

## Important

- Always confirm the version number and release notes with the user before committing
- Never force-push or delete tags
- If anything fails mid-process, explain what happened and what manual steps (if any) are needed to recover
- The `make dmg` step takes a while — let it run with a timeout of 600 seconds
