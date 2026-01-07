# Fuzzy Search Plan

## Goals
- Add fuzzy search to the TUI picker lists for theme, waybar, and starship.
- Keep selection stable while filtering and avoid breaking previews.
- Document new keybindings and add coverage for filter logic.

## Implementation Steps
1. Add search state to each picker.
   - Track `search_query`, `search_active`, `filtered_indices`, and `last_selected`.
   - Rebuild filtered indices from the full item list on query or list changes.

2. Filter and sort items with fuzzy matching.
   - Use a fuzzy matcher to score item labels.
   - Sort by score, then by label for stable ordering.
   - Map filtered list indices back to the original item index.

3. Update the TUI list rendering.
   - Render list items from the filtered indices.
   - Show the active search query in the list title.
   - If no matches, show a simple "No matches" preview message.

4. Wire up keyboard handling.
   - `/` starts search (clears existing query).
   - `Esc` exits search and clears the query.
   - `Enter` exits search and keeps the query.
   - `Backspace` removes one character.
   - `Ctrl+u` clears the query.

5. Selection and preview behavior.
   - Preserve the last selected item if it still matches.
   - Use the mapped item index for previews and selections.

6. Tests and docs.
   - Add unit tests for the fuzzy filtering and ordering logic.
   - Update the browse usage/help text to mention search keys.
