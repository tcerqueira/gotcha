with
  deleted_api_secret as (
    delete from public.api_secret
    where
      key = '4BdwFU84HLqceCQbE90+U5mw7f0erayega3nFOYvp1T5qXd8IqnTHJfsh675Vb2q'
    returning
    console_id
  ),
  deleted_console as (
    delete from public.console
    where
      id = (
        select
        console_id
        from
          deleted_api_secret
      )
      returning id
  )
select
  *
from
  deleted_console;
