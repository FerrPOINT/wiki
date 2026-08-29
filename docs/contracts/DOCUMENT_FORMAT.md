# Document Format Contract - Wiki

## 1. Source Format

Markdown is the canonical editable format for MVP.

## 2. Rendering

- Parse Markdown with `comrak`.
- Render HTML server-side or in controlled frontend preview.
- Sanitize rendered HTML with `ammonia`.
- Store source Markdown and sanitized HTML separately.

## 3. Supported Features

- headings;
- paragraphs;
- links;
- tables;
- fenced code blocks;
- task lists;
- relative links to Wiki documents;
- mentions as plain text in MVP.

## 4. Safety

- Raw unsafe HTML is removed.
- Script/event attributes are blocked.
- External links may get `rel="noopener noreferrer"`.
- Attachments are linked by attachment ID, not raw storage key.
