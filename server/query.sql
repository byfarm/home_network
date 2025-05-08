INSERT INTO
    data (location_id, value, measurand, units)
SELECT
    location.id,
    ?2,
    ?3,
    ?4
FROM
    location
WHERE
    location.name = ?1
LIMIT
    1
