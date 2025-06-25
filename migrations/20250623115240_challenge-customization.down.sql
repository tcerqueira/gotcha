-- Drop challenge customization table
drop table public.challenge_customization;

-- Remove default_logo_url column and rename columns back to original names
alter table public.challenge
drop column if exists default_logo_url,
drop column if exists label;

alter table public.challenge
rename column default_width to width;

alter table public.challenge
rename column default_height to height;

-- Update constraint names back to original names
alter table public.challenge
drop constraint challenge_default_width_range,
drop constraint challenge_default_height_range,
add constraint width_positive check (width > 0),
add constraint height_positive check (height > 0);
