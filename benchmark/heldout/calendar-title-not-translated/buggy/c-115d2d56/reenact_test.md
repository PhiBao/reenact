---
mode: testing
url: http://localhost:16208/auth
headless: true
max_steps: 40
tags: [reenact, issue-5578]
---

# Change the translation message of calendar title immediately when change language

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:16208/.
Stay on localhost:16208 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Open the home page and note the calendar title (month and year) in the sidebar
2. Open the Settings page (My Account / Preferences)
3. Find the language selector in the preferences
4. Select a different language from the language selector, such as Chinese or another non-English language
5. Return to the home page and look at the calendar title
The reporter says the correct behavior is that The calendar title month and year text is displayed in the newly selected language immediately, without refreshing the page. Verify that The calendar title month and year text is displayed in the newly selected language immediately, without refreshing the page.
