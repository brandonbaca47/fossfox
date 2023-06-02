# [Fossfox](https://fossfox.com/)

## What Is Fossfox

[Fossfox](https://fossfox.com/) is a **job board for software engineers** 👩‍💻👨‍💻

It lists companies whose products are open-source (or are a major contributor to OSS). Our goal is to get software engineers better jobs.

[![Fossfox](static/img/fossfox.webp)](https://fossfox.com/)

## Why Post On Fossfox

1. Posting on Fossfox is free
1. Our audience is tech-savvy software engineers
1. Use our tech-specific traffic to get more relevant candidates

## How To Post

There are 2 ways to post new jobs on Fossfox:

### Manually via the template

1. Clone this repo
1. Use [data/SAMPLE_COMPANY_TEMPLATE.json](data/SAMPLE_COMPANY_TEMPLATE.json) to create a new file
1. Fill it out with your company details & engineering roles
1. Rename your new file to a relevant slug (eg: "My Company" → `my-company.json`)
1. Place your new file in the companies folder (eg: `data/companies/m/my-company.json`)
1. [Create a pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request) so we can merge it

### Using a CLI wizard

1. Clone this repo
1. [Install Rust](https://www.rust-lang.org/tools/install) if you don't have it
1. Run `cargo run` and follow the wizard

The job posts will automatically appear on the homepage after your PR will be accepted.

## FAQs

<details>
  <summary>How much does it cost?</summary>
  Free.
</details>

<details>
  <summary>Why is it free? What's the catch?</summary>
  Our homepage has a limit on how many positions it shows at once. For more visibility we are offering <a href="mailto:support@fossfox.com">sponsored posts</a>.
</details>

<details>
  <summary>How long will Fossfox stay free?</summary>
  Forever.
</details>

<details>
  <summary>How many jobs can I post?</summary>
  Unlimited, as long as they're all part of your engineering team.
</details>

<details>
  <summary>How long do the posts stay up?</summary>
  30 days. After that you can update the timestamp to extend for another 30 days.
</details>

<details>
  <summary>Can I post non-engineering related job posts?</summary>
  No. This job board is for tech-only positions.
</details>
