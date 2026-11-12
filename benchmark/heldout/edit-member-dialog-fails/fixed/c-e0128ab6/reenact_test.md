---
mode: testing
url: http://localhost:16211/auth
headless: true
max_steps: 40
tags: [reenact, issue-5564]
---

# v0.26.0 Unable to update member - Update user: [invalid_argument] rpc error: code = InvalidArgument

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:16211/setting#member.
Stay on localhost:16211 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Open the Settings page and go to the Member section
2. Click the button to create a new member
3. Fill in a username and password for the new member and confirm creation
4. In the member list, open the edit option for the newly created member
5. Change the username or password of the member
6. Click Save
The reporter says the correct behavior is that The member edit is saved and no error message containing 'invalid user name' is visible on the page. Verify that The member edit is saved and no error message containing 'invalid user name' is visible on the page.
