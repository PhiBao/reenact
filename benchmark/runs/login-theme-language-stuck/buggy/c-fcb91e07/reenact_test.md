---
mode: testing
url: http://localhost:15306/auth
headless: true
max_steps: 40
tags: [reenact, issue-5440]
---

# Bug: Login screen Theme and Language settings stuck (v0.25.3)

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the user is signed in and the main memo page is visible.

## Reproduce the report
Open http://localhost:15306/auth.
Stay on localhost:15306 for the whole run and do not open any other website.
Before the main steps, make sure of this: the browser session is logged out so the Sign In page is shown; the instance runs Memos v0.25.3 (Docker image neosmemo/memos:stable).
Do these in order:
1. Open the Sign In page without logging in
2. Click the Language dropdown at the bottom of the page
3. Select a different language from the dropdown list
4. Click the Theme toggle button (Sun/Moon icon)
The reporter says the correct behavior is that after selecting a different language the Sign In page text is shown in that language, and after clicking the theme toggle the page switches between light and dark theme. Verify that after selecting a different language the Sign In page text is shown in that language, and after clicking the theme toggle the page switches between light and dark theme.
