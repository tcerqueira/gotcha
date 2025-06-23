-- Add default columns to challenge table and rename existing columns
alter table public.challenge
rename column width to default_width;

alter table public.challenge
rename column height to default_height;

alter table public.challenge
add column default_logo_url character varying null;

-- Update constraint names to match new column names
alter table public.challenge
drop constraint width_positive,
drop constraint height_positive,
add constraint challenge_default_width_range check (default_width > 0),
add constraint challenge_default_height_range check (default_height > 0);

-- Create challenge customization table (multiple per console)
create table public.challenge_customization (
    id uuid not null default gen_random_uuid (),
    console_id uuid not null,
    challenge_url character varying not null,
    width smallint not null default 360,
    height smallint not null default 500,
    small_width smallint not null default 360,
    small_height smallint not null default 500,
    logo_url character varying null,
    created_at timestamp
    with
        time zone not null default now (),
        constraint challenge_customization_pkey primary key (id),
        constraint challenge_customization_console_id_fkey foreign key (console_id) references console (id) on delete cascade,
        constraint challenge_customization_challenge_url_fkey foreign key (challenge_url) references challenge (url) on delete cascade,
        constraint challenge_customization_width_positive check (width > 0),
        constraint challenge_customization_height_positive check (height > 0),
        constraint challenge_customization_small_width_positive check (small_width > 0),
        constraint challenge_customization_small_height_positive check (small_height > 0),
        constraint challenge_customization_console_challenge_unique unique (console_id, challenge_url)
);

-- Copy existing challenge data to challenge_customization for all existing consoles
-- This creates challenge_customization records for all existing challenges for all consoles
insert into
    public.challenge_customization (
        console_id,
        challenge_url,
        width,
        height,
        created_at
    )
select
    c.id,
    ch.url,
    ch.default_width,
    ch.default_height,
    ch.created_at
from
    public.challenge ch
    cross join public.console c;
