-- Change names
alter table public.api_key
drop constraint api_key_console_id_fkey;

alter table public.api_key
drop constraint api_key_pkey;

alter table public.api_key
rename to api_secret;

alter table public.api_secret
rename column site_key to key;

alter table public.api_secret add constraint api_secret_pkey primary key (key);

alter table public.api_secret add constraint api_secret_console_id_fkey foreign key (console_id) references public.console (id) on delete cascade;

-- Console support
alter table public.api_secret
drop column label,
drop column secret;

alter table public.console
drop column user_id,
alter column label
drop default;
