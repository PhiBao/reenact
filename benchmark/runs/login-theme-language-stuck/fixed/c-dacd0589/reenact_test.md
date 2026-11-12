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
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Open the Sign In page while signed out
2. Click the Language dropdown at the bottom of the page
3. Select a different language from the dropdown
4. Click the Theme toggle button (Sun/Moon icon)
The reporter says the correct behavior is that After selecting a different language, the Sign In page text is displayed in that language, and after clicking the theme toggle the page switches between light and dark appearance. Verify that After selecting a different language, the Sign In page text is displayed in that language, and after clicking the theme toggle the page switches between light and dark appearance.
