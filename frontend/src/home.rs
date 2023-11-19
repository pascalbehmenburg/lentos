// pub async fn side_bar_dropdown_item<'a>(title: &'a str, dropdown_items: &'a Vec<(&str, &str)>) -> Markup {
//     html! {
//         li {
//             details class="group [&_summary::-webkit-details-marker]:hidden";
//             summary class="flex cursor-pointer items-center justify-between rounded-lg px-4 py-2 text-gray-500 hover:bg-gray-100 hover:text-gray-700" {
//                 span class="text-sm font-medium" { (title) }
//                 span class="shrink-0 transition duration-300 group-open:-rotate-180" {
//                     svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor" {
//                         path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd";
//                     }
//                 }
//             }
//             ul class="mt-2 space-y-1 px-4" {
//                 @for item in dropdown_items {
//                     (side_bar_item(item.0, item.1).await)
//                 }
//             }
//         }
//     }
// }





// pub async fn side_bar_user_info<'a>(user_name: &'a str, user_link: &'a str, user_image: &'a str) -> Markup {
//     html! {
//         div class="sticky inset-x-0 bottom-0 border-t border-gray-100" {
//             a href=(user_link) class="flex items-center gap-2 bg-white p-4 hover:bg-gray-50" {
//                 img alt="" src=(user_image) class="h-10 w-10 rounded-full object-cover";
//                 div {
//                     p class="text-xs" {
//                         strong class="block font-medium" { (user_name) }
//                         span { (user_link) }
//                     }
//                 }
//             }
//         }
//     }
// }

// pub async fn side_bar_logo(logo: &Markup) -> Markup {
//     html! {
//             span class="grid h-10 w-32 place-content-center rounded-lg bg-gray-100 text-xs text-gray-600" {
//                 (logo)
//             }
//     }
// }

// pub async fn main_view(user: &User) -> Markup {
//     html! {
//         div class="grid grid-cols-1 gap-4 lg:grid-cols-3 lg:gap-8" {
//             div class="h-32 rounded-lg bg-gray-200" {
//                 div class="flex h-screen flex-col justify-between border-e bg-white" {
//                     div class="px-4 py-6" {
//                         (side_bar_logo(&html! { "Logo" }).await)
//                         ul class="mt-6 space-y-1" {
//                             (side_bar_item("Today", "/filter/today").await)
//                             (side_bar_item("Upcoming", "/filter/upcoming").await)
//                             (side_bar_item("Anytime", "/filter/anytime").await)
//                             (side_bar_item("Someday", "/filter/someday").await)
//                         }
//                     }
//                     (side_bar_user_info(&user.name, "", "").await)
//                 }
//             }
//             div class="h-32 rounded-lg bg-gray-200 lg:col-span-2" {
//                 div class="flex h-screen flex-col justify-between border-e bg-white" {
//                     div hx-get="/component/todo/list" hx-trigger="load delay:500ms" {
//                         img alt="Result loading..." class="htmx-indicator" width="150" src="/img/bars.svg";
//                     }
//                 }
//             }
//         }

//     }
// }

// pub async fn side_bar_item<'a>(title: &'a str, href: &'a str) -> Markup {
//     html! {
//         li {
//             a href=(href) class="block rounded-lg bg-gray-100 px-4 py-2 text-sm font-medium text-gray-700" {
//                 (title)
//             }
//         }
//     }
// }

// fn SideBarItem(cx: Scope) -> Element {
//     render! {
//         li {
//             a href=()
//         }
//     }
// }