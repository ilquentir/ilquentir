-- Add up migration script here
ALTER TABLE poll_answers ADD COLUMN selected_value_text VARCHAR(100);

UPDATE poll_answers SET selected_value_text = (
    CASE
        WHEN selected_value = 0 THEN '+2 (супер!)'
        WHEN selected_value = 1 THEN '+1'
        WHEN selected_value = 2 THEN '0'
        WHEN selected_value = 3 THEN '-1'
        WHEN selected_value = 4 THEN '-2 (отвратительно)'
    END
)
FROM polls
WHERE
    poll_answers.poll_tg_id = polls.tg_id
    AND polls.kind = 'how_was_your_day';

UPDATE poll_answers SET selected_value_text = (
    CASE
        WHEN selected_value = 0 THEN 'Shortness of breath'
        WHEN selected_value = 1 THEN 'Itching'
        WHEN selected_value = 2 THEN 'Bloating'
        WHEN selected_value = 3 THEN 'Nope, nothing :)'
    END
)
FROM polls
WHERE
    poll_answers.poll_tg_id = polls.tg_id
    AND polls.kind = 'food_allergy';

ALTER TABLE poll_answers ALTER COLUMN selected_value_text SET NOT NULL;
