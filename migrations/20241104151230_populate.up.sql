with
  config as (insert into public.configuration (LABEL) values ('test') returning id)
insert into
  public.api_key (KEY, config)
values
  (
    '4BdwFU84HLqceCQbE90+U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q',
    (select id from config)
  );
