use leptos::*;

mod view;

use self::view::*;

#[component]
pub fn App() -> impl IntoView {

    // ---     --- //
    //   signals   //
    // ---     --- //


    // ---     --- //
    //   actions   //
    // ---     --- //

    // ---     --- //
    //  Callbacks  //
    // ---     --- //


    // ---     --- //
    //  Resources  //
    // ---     --- //


    // ---     --- //
    //     View    //
    // ---     --- //

    view! {
        <MainView/>
    }
}
