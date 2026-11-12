---
mode: testing
url: http://localhost:15318/auth
headless: true
max_steps: 40
tags: [reenact, issue-5640]
---

# Comment reactions and comment deletion don’t update the UI until page refresh

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15318/.
Stay on localhost:15318 for the whole run and do not open any other website.
Do these in order:
1. Create a new memo with the text "Test memo" and save it
2. Click the new memo to open its detail page
3. In the comment input on the memo detail page, type "Test comment"
4. Click the button to submit the comment
5. On the comment, open the reaction selector in the comment header
6. Select a reaction emoji (for example 👍)
7. Observe the comment without refreshing the page
8. Click the action menu (⋮) on the comment
9. Click "Delete"
10. Click the confirm button in the confirmation dialog
11. Observe the comments list without refreshing the page
The reporter says the correct behavior is that After selecting a reaction, the reaction appears on the comment immediately without a page refresh; after confirming deletion, the comment "Test comment" is no longer visible in the comments list without a page refresh. Verify that After selecting a reaction, the reaction appears on the comment immediately without a page refresh; after confirming deletion, the comment "Test comment" is no longer visible in the comments list without a page refresh.
