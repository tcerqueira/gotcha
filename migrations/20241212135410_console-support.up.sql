alter table public.api_secret
drop constraint api_secret_pkey;

alter table public.api_secret
add column label character varying default null,
add column secret character varying,
add column id uuid not null default gen_random_uuid ();

update public.api_secret
set
    secret = key;

alter table public.api_secret
alter column secret
set
    not null;

alter table public.api_secret add constraint api_secret_pkey primary key (id);

alter table public.console
add column user_id character varying,
alter column label
set default null;

update public.console
set
    user_id = 'demo';

alter table public.console
alter column user_id
set
    not null;
