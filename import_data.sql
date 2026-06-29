-- Clear existing data first (child tables first)
DELETE FROM email_verifications;
DELETE FROM captcha_challenges;
DELETE FROM repositories;
DELETE FROM users;

-- Users (1 row, original table structure matches)
INSERT INTO "public"."users" VALUES ('c5e3d293-d0e3-4a4d-a8f2-d335c5f372a1', 'BlackWinds', 'blackwinds@foxmail.com', '$argon2id$v=19$m=19456,t=2,p=1$A98GWQg0ATai8gm02qm5EA$vMbisU//3tC0l5iEALL4EmdWYBR6xTyTW17kOhuNYM4', 'BlackWinds', NULL, NULL, 't', 'f', '2026-06-14 19:58:44.520287+08', '2026-06-14 19:58:44.520287+08');

-- Email verification (1 row)
INSERT INTO "public"."email_verifications" VALUES ('e2455dcf-3f04-4947-a35f-80cf2662b5a9', 'c5e3d293-d0e3-4a4d-a8f2-d335c5f372a1', 'b673af70-5317-4f0d-91a6-b05050bdeab9', '2026-06-15 19:58:44.52188+08', 'f', '2026-06-14 19:58:44.522247+08');

-- Repositories (2 rows, added max_file_size_mb=100, enable_notifications=true for new columns)
INSERT INTO "public"."repositories" VALUES ('86dcb185-cd22-43bc-86d8-51659322e9d6', 'user', 'c5e3d293-d0e3-4a4d-a8f2-d335c5f372a1', 'GitRust', '', 'main', 'f', 'f', 'f', '2026-06-14 21:12:13.41849+08', '2026-06-14 21:12:13.41849+08', 100, 't');
INSERT INTO "public"."repositories" VALUES ('43908a0a-4f89-47cb-a6c6-64f7cef7d8db', 'user', 'c5e3d293-d0e3-4a4d-a8f2-d335c5f372a1', 'test', '', 'main', 'f', 'f', 'f', '2026-06-14 21:12:53.686201+08', '2026-06-14 21:12:53.686201+08', 100, 't');

-- Captcha challenges (23 rows, original table matches)
INSERT INTO "public"."captcha_challenges" VALUES ('ebe9557e-4fc6-4822-8de0-7f17a91a65e5', '4fe9f923-44fe-4dba-9af4-c2d3677abfb7', 'wes64', '2026-06-14 20:01:18.473849+08', '2026-06-14 19:51:18.475161+08');
INSERT INTO "public"."captcha_challenges" VALUES ('cff20b97-f0db-4b25-8b9b-57f1add0de92', 'fd2b70ef-fcb2-40a6-bc45-c672d8167683', 'kr2hv', '2026-06-14 20:01:47.32194+08', '2026-06-14 19:51:47.322063+08');
INSERT INTO "public"."captcha_challenges" VALUES ('e6c78100-7f53-48f2-b489-ecb1af672331', '5a877b54-6225-49f3-afd7-0d93116064f7', 'spwun', '2026-06-14 20:01:51.804158+08', '2026-06-14 19:51:51.804277+08');
INSERT INTO "public"."captcha_challenges" VALUES ('b92f73cc-f367-4bb1-acbc-590cc46f687d', 'a87d4493-9dae-4ecd-ae5e-f5be7882e560', 'emcsj', '2026-06-14 20:01:52.57127+08', '2026-06-14 19:51:52.571449+08');
INSERT INTO "public"."captcha_challenges" VALUES ('3ebd605d-771e-41d6-acbc-5ed39eb2f004', '9ffe2278-34d1-444a-941d-78ea8e632f07', 'y7k6u', '2026-06-14 20:01:56.851061+08', '2026-06-14 19:51:56.851198+08');
INSERT INTO "public"."captcha_challenges" VALUES ('920415dd-681a-4cf8-8f09-248f19dd59d8', 'cfa60429-b62b-4372-b1c1-a5244d6e1369', 'pkk34', '2026-06-14 20:01:59.858511+08', '2026-06-14 19:51:59.858668+08');
INSERT INTO "public"."captcha_challenges" VALUES ('4079ae25-664c-4ed2-afa8-499af3644402', '3c9c0459-f66d-402a-a9a9-39c8f347af7c', 'wtpju', '2026-06-14 20:05:56.487137+08', '2026-06-14 19:55:56.487938+08');
INSERT INTO "public"."captcha_challenges" VALUES ('92fd04e1-7239-4806-8d4b-1a868a25b0ce', '707c3519-f67e-4783-9ed8-03cd843751e8', 'amusx', '2026-06-14 20:05:58.037571+08', '2026-06-14 19:55:58.037709+08');
INSERT INTO "public"."captcha_challenges" VALUES ('f00a1690-6287-4eae-96c1-29f7409ef5af', 'c4495736-e7a8-4a2c-9b23-76d241ce160c', 'pdb6t', '2026-06-14 20:05:59.393229+08', '2026-06-14 19:55:59.393398+08');
INSERT INTO "public"."captcha_challenges" VALUES ('10ebf673-b9ff-429a-a2eb-cbab71fbd8e6', 'd1c8aae5-3476-431d-b604-0d98d8f7b2b0', 'kxezb', '2026-06-14 20:06:00.764505+08', '2026-06-14 19:56:00.764658+08');
INSERT INTO "public"."captcha_challenges" VALUES ('aae4e5e3-fdfb-44e8-ba86-28314d44e21a', '8a030cff-793d-409d-82b9-75e58d55bfbc', 'dydtw', '2026-06-14 20:06:02.13327+08', '2026-06-14 19:56:02.133393+08');
INSERT INTO "public"."captcha_challenges" VALUES ('3c82ae01-6696-4382-9744-948386c5b1a1', 'c5540e3b-cc39-47d9-b2d8-9eb73291c20a', 'tmgrx', '2026-06-14 20:06:02.887528+08', '2026-06-14 19:56:02.887681+08');
INSERT INTO "public"."captcha_challenges" VALUES ('2ff632d7-8a15-42d5-bfea-eaf15b541131', '1f258513-9a28-4a04-904e-d02638d9b016', 'zg2ad', '2026-06-14 20:06:03.979413+08', '2026-06-14 19:56:03.979584+08');
INSERT INTO "public"."captcha_challenges" VALUES ('12ae442e-6d55-4b84-ac31-123a03b71575', 'c80c0d01-5e83-4fa2-984d-794097a8ca43', '5kkzc', '2026-06-14 20:06:32.652355+08', '2026-06-14 19:56:32.652528+08');
INSERT INTO "public"."captcha_challenges" VALUES ('1295db86-2236-4205-92f3-bd96a0158e7a', '34f14d2a-d05a-4afe-ab1d-cc759d2bdb9e', 'pbxwz', '2026-06-14 20:07:42.460932+08', '2026-06-14 19:57:42.462134+08');
INSERT INTO "public"."captcha_challenges" VALUES ('d7fed74b-6eee-4b73-a4e2-b71c34c14963', 'd76f009b-21f9-497c-a0a0-951a295cc12b', 'u39fr', '2026-06-14 20:07:44.586152+08', '2026-06-14 19:57:44.586287+08');
INSERT INTO "public"."captcha_challenges" VALUES ('920c6369-e6c4-4513-b1f0-4f747ba85a3c', '3f2a4c24-5625-4d55-a675-d2f4eb15c5de', 'jnsjq', '2026-06-14 20:07:45.069938+08', '2026-06-14 19:57:45.070068+08');
INSERT INTO "public"."captcha_challenges" VALUES ('a1b11fbb-f25e-46e5-9bd8-eac9dbae582a', '9f2aa988-0074-4ff7-8c9a-8253361e0c14', '3fr7y', '2026-06-14 20:07:45.420318+08', '2026-06-14 19:57:45.42048+08');
INSERT INTO "public"."captcha_challenges" VALUES ('cbe7fa28-1d4f-4834-b2bc-3946a3a3135d', 'd2ef4f25-4eff-4b91-a6aa-3e0e11fbcf70', 'uxfkc', '2026-06-14 20:07:45.695335+08', '2026-06-14 19:57:45.695457+08');
INSERT INTO "public"."captcha_challenges" VALUES ('da13754c-30b7-4cf7-9715-cdcf7aa72912', '94877a51-2db8-4517-86ba-8ba3f269fa15', '67cyu', '2026-06-14 20:07:46.094536+08', '2026-06-14 19:57:46.094758+08');
INSERT INTO "public"."captcha_challenges" VALUES ('5bb7e77a-9757-457b-9fa3-a884d3e85fcc', 'cdabbbff-3a6f-4b50-ac62-7594e23eb41b', 'bu23p', '2026-06-14 20:14:32.571615+08', '2026-06-14 20:04:32.572352+08');
INSERT INTO "public"."captcha_challenges" VALUES ('6965279f-46dc-49fe-982a-19ae31aada11', 'eb4de0fd-1a6a-43f9-800b-471d91c7a903', '9fegu', '2026-06-14 21:59:39.935568+08', '2026-06-14 21:49:39.936047+08');
INSERT INTO "public"."captcha_challenges" VALUES ('40ce594d-a872-43de-ab8e-298b4be7aa0e', '7f3ea31f-b617-4ab8-b36f-deec8a26d0f9', 'wwprd', '2026-06-14 22:07:26.17021+08', '2026-06-14 21:57:26.170763+08');
