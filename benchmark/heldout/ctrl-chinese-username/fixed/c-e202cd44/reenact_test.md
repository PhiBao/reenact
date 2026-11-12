---
mode: testing
url: http://localhost:16219/auth/signup
headless: true
max_steps: 40
tags: [reenact, issue-5594]
---

# Username Registration Does Not Support Chinese Characters

## Reproduce the report
Open http://localhost:16219/auth/signup.
Stay on localhost:16219 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Open the sign-up page
2. Click the username field
3. Type "隧道" into the username field
4. Click the password field
5. Type "TestPass123!" into the password field
6. Click the Sign up button
The reporter says the correct behavior is that No "invalid username" error is shown, and the registration succeeds with the username 隧道 so the user leaves the sign-up page. Verify that No "invalid username" error is shown, and the registration succeeds with the username 隧道 so the user leaves the sign-up page.
