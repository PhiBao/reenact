---
mode: testing
url: http://localhost:15303/auth
headless: true
max_steps: 40
tags: [reenact, issue-5575]
---

# The nested checklist cannot display levels correctly

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the user is signed in and the main memo page is visible.

## Reproduce the report
Open http://localhost:15303/.
Stay on localhost:15303 for the whole run and do not open any other website.
Do these in order:
1. Open the memo editor on the home page to create a new memo
2. Type the line '- [ ] task1' and press Enter
3. Type the line ' - [ ] task1.1' (indented by 4 spaces) and press Enter
4. Type the line ' - [ ] task 1.1.1' (indented by 8 spaces) and press Enter
5. Type the line ' - [ ] task1.2 ' (indented by 4 spaces) and press Enter
6. Type the line '- [ ] task2' with no indentation
7. Click the Save button to publish the memo
8. View the rendered memo in the memo list
9. Click the checkbox of 'task1.1' in the rendered memo
The reporter says the correct behavior is that the rendered memo shows task1.1 and task1.2 indented under task1, task 1.1.1 indented under task1.1, and clicking the checkbox of task1.1 checks only task1.1. Verify that the rendered memo shows task1.1 and task1.2 indented under task1, task 1.1.1 indented under task1.1, and clicking the checkbox of task1.1 checks only task1.1.
