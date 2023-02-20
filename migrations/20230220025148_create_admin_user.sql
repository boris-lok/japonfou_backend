-- Add migration script here

insert into users (id, username, password_hash)
values ('44e1918b-c79f-48de-a68a-eb6e0f667132', 'boris',
        '$argon2id$v=19$m=15000,t=2,p=1$b/NgBglStilwrAKUAjF+Tw$WBPVAZ6MLsGLElOYYbctM5w68AHMvPVlVSXHow88Ywg')