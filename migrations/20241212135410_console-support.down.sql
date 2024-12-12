alter table public.api_secret
drop constraint api_secret_pkey;

alter table public.api_secret
drop column label,
drop column secret,
drop column id;

alter table public.api_secret add constraint api_secret_pkey primary key (key);

alter table public.console
drop column user_id,
alter column label
drop default;
