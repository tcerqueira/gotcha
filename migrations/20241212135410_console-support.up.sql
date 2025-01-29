-- Console support
alter table public.api_secret
add column label character varying default null,
add column secret character varying;

update public.api_secret
set
    secret = key;

alter table public.api_secret
alter column secret
set
    not null;

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

-- Change names
alter table public.api_secret
drop constraint api_secret_console_id_fkey;

alter table public.api_secret
drop constraint api_secret_pkey;

alter table public.api_secret
rename to api_key;

alter table public.api_key
rename column key TO site_key;

alter table public.api_key add constraint api_key_pkey primary key (site_key);

alter table public.api_key add constraint api_key_console_id_fkey foreign key (console_id) references public.console (id) on delete cascade;

-- Unique secret
alter table public.api_key add constraint api_key_secret_unique unique (secret);
