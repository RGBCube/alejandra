pub(crate) fn rule(
    build_ctx: &crate::builder::BuildCtx,
    node: &rnix::SyntaxNode,
) -> std::collections::LinkedList<crate::builder::Step> {
    let mut steps = std::collections::LinkedList::new();

    let mut children = crate::children::Children::new(build_ctx, node);

    let items_count = node
        .children_with_tokens()
        .skip_while(|element| element.kind() != rnix::SyntaxKind::TOKEN_L_BRACE)
        .take_while(|element| element.kind() != rnix::SyntaxKind::TOKEN_R_BRACE)
        .filter(|element| {
            matches!(
                element.kind(),
                rnix::SyntaxKind::NODE_ATTRPATH_VALUE
                    | rnix::SyntaxKind::NODE_INHERIT
                    | rnix::SyntaxKind::NODE_INHERIT_FROM
                    | rnix::SyntaxKind::TOKEN_COMMENT
            )
        })
        .count();

    let vertical = items_count > 1
        || children.has_comments()
        || children.has_newlines()
        || build_ctx.vertical;

    // rec
    let child = children.peek_next().unwrap();
    if let rnix::SyntaxKind::TOKEN_REC = child.kind() {
        steps.push_back(crate::builder::Step::Format(child));
        children.move_next();

        if let rnix::SyntaxKind::TOKEN_COMMENT
        | rnix::SyntaxKind::TOKEN_WHITESPACE =
            children.peek_next().unwrap().kind()
        {
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        } else {
            steps.push_back(crate::builder::Step::Whitespace);
        }
    }

    // /**/
    children.drain_trivia(|element| match element {
        crate::children::Trivia::Comment(text) => {
            steps.push_back(crate::builder::Step::Comment(text));
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        }
        crate::children::Trivia::Whitespace(_) => {}
    });

    // {
    let child = children.get_next().unwrap();
    steps.push_back(crate::builder::Step::Format(child));
    if vertical {
        steps.push_back(crate::builder::Step::Indent);
    }

    let mut item_index: usize = 0;
    let mut inline_next_comment = false;

    loop {
        // /**/
        children.drain_trivia(|element| match element {
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

                if newlines > 1 && item_index > 0 && item_index < items_count {
                    steps.push_back(crate::builder::Step::NewLine);
                }

                inline_next_comment = newlines == 0;
            }
        });

        if let Some(child) = children.peek_next() {
            if let rnix::SyntaxKind::TOKEN_R_BRACE = child.kind() {
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

    // }
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
