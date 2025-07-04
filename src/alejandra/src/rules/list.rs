pub(crate) fn rule(
    build_ctx: &crate::builder::BuildCtx,
    node: &rnix::SyntaxNode,
) -> std::collections::LinkedList<crate::builder::Step> {
    let mut steps = std::collections::LinkedList::new();

    let mut children = crate::children::Children::new(build_ctx, node);

    let items_count = node
        .children_with_tokens()
        .filter(|element| {
            !matches!(element.kind(), rnix::SyntaxKind::TOKEN_WHITESPACE)
        })
        .count()
        - 2;

    let vertical = children.has_comments()
        || children.has_newlines()
        || build_ctx.vertical;

    // [
    let child = children.get_next().unwrap();
    steps.push_back(crate::builder::Step::Format(child));
    if vertical {
        steps.push_back(crate::builder::Step::Indent);
    }

    let mut item_index: usize = 0;
    let mut inline_next_comment = false;

    loop {
        // /**/
        children.drain_trivia(|element| {
            match element {
                crate::children::Trivia::Comment(text) => {
                    if inline_next_comment && text.starts_with('#') {
                        steps.push_back(crate::builder::Step::Whitespace);
                    } else {
                        steps.push_back(crate::builder::Step::NewLine);
                        steps.push_back(crate::builder::Step::Pad);
                    }

                    steps.push_back(crate::builder::Step::Comment(text));
                    item_index += 1;
                    inline_next_comment = false;
                }
                crate::children::Trivia::Whitespace(text) => {
                    let newlines = crate::utils::count_newlines(&text);

                    if newlines > 1
                        && item_index > 0
                        && item_index < items_count
                    {
                        steps.push_back(crate::builder::Step::NewLine);
                    }

                    inline_next_comment = newlines == 0;
                }
            };
        });

        if let Some(child) = children.peek_next() {
            let child_kind = child.kind();

            if let rnix::SyntaxKind::TOKEN_R_BRACK = child_kind {
                break;
            }

            // item
            item_index += 1;
            if vertical {
                steps.push_back(crate::builder::Step::NewLine);
                steps.push_back(crate::builder::Step::Pad);
                steps.push_back(crate::builder::Step::FormatWider(child));
            } else {
                steps.push_back(crate::builder::Step::Whitespace);
                steps.push_back(crate::builder::Step::Format(child));
            }

            children.move_next();
            inline_next_comment = true;
        }
    }

    // ]
    let child = children.get_next().unwrap();
    if vertical {
        steps.push_back(crate::builder::Step::Dedent);
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
    } else if items_count > 0 {
        steps.push_back(crate::builder::Step::Whitespace);
    }
    steps.push_back(crate::builder::Step::Format(child));

    steps
}
