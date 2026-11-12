---
mode: testing
url: http://localhost:15319/auth
headless: true
max_steps: 40
tags: [reenact, issue-5640]
---

# Comment reactions and comment deletion don’t update the UI until page refresh

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15319/.
Stay on localhost:15319 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Create a new memo with the text 'test memo' and save it
2. Open the newly created memo to view its detail page
3. In the comment box on the memo detail page, type 'test comment' and submit the comment
4. On the 'test comment' comment, open the reaction selector in the comment header
5. Choose a reaction (for example the first emoji shown) in the reaction selector
6. Observe the comment card without refreshing the page
7. Click the action menu (⋮) on the 'test comment' comment
8. Click 'Delete'
9. Confirm the deletion in the confirmation dialog
The reporter says the correct behavior is that After the deletion is confirmed, the 'test comment' comment is no longer visible in the comments list without refreshing the page (and after choosing a reaction, the chosen reaction is visible on the comment immediately). Verify that After the deletion is confirmed, the 'test comment' comment is no longer visible in the comments list without refreshing the page (and after choosing a reaction, the chosen reaction is visible on the comment immediately).
