---
mode: testing
url: http://localhost:15307/auth
headless: true
max_steps: 40
tags: [reenact, issue-5440]
---

# Bug: Login screen Theme and Language settings stuck (v0.25.3)

## Reproduce the report
Open http://localhost:15307/auth.
Stay on localhost:15307 for the whole run and do not open any other website.
Do these in order:
1. Open the Sign In page while signed out
2. Click the language dropdown at the bottom of the Sign In page
3. Select a different language from the dropdown
4. Click the theme toggle button (sun/moon icon)
The reporter says the correct behavior is that After selecting a different language, the Sign In page text is displayed in that language, and after clicking the theme toggle the page switches between light and dark appearance immediately. Verify that After selecting a different language, the Sign In page text is displayed in that language, and after clicking the theme toggle the page switches between light and dark appearance immediately.
