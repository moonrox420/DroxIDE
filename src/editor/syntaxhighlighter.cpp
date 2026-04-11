// src/editor/syntaxhighlighter.cpp
#include "syntaxhighlighter.h"

SyntaxHighlighter::SyntaxHighlighter(QTextDocument *document, const QString &language)
    : QSyntaxHighlighter(document)
{
    // Keyword format
    keywordFormat.setForeground(Qt::darkBlue);
    keywordFormat.setFontWeight(QFont::Bold);
    
    // Class format
    classFormat.setForeground(Qt::darkMagenta);
    classFormat.setFontWeight(QFont::Bold);
    
    // Single line comment
    singleLineCommentFormat.setForeground(Qt::darkGreen);
    
    // Multi-line comment
    multiLineCommentFormat.setForeground(Qt::darkGreen);
    
    // Quotation
    quotationFormat.setForeground(Qt::darkRed);
    
    // Function
    functionFormat.setForeground(Qt::blue);
    
    // Rust keywords
    if (language == QStringLiteral("rust")) {
        QStringList rustKeywords;
        rustKeywords << QStringLiteral("\\bfn\\b") << QStringLiteral("\\blet\\b") << QStringLiteral("\\bmut\\b") << QStringLiteral("\\bif\\b")
                     << QStringLiteral("\\belse\\b") << QStringLiteral("\\bwhile\\b") << QStringLiteral("\\bloop\\b")
                     << QStringLiteral("\\bstruct\\b") << QStringLiteral("\\benum\\b") << QStringLiteral("\\bimpl\\b") << QStringLiteral("\\btrait\\b")
                     << QStringLiteral("\\bpub\\b") << QStringLiteral("\\bprivate\\b") << QStringLiteral("\\bself\\b") << QStringLiteral("\\buse\\b")
                     << QStringLiteral("\\bmod\\b") << QStringLiteral("\\basync\\b") << QStringLiteral("\\bawait\\b") << QStringLiteral("\\breturn\\b");

        for (const QString &pattern : rustKeywords) {
            HighlightingRule rule;
            rule.pattern = QRegularExpression(pattern);
            rule.format = keywordFormat;
            highlightingRules.append(rule);
        }
    }

    // Python keywords
    else if (language == QStringLiteral("python")) {
        QStringList pythonKeywords;
        pythonKeywords << QStringLiteral("\\bdef\\b") << QStringLiteral("\\bclass\\b") << QStringLiteral("\\bif\\b") << QStringLiteral("\\belse\\b")
                       << QStringLiteral("\\belif\\b") << QStringLiteral("\\bfor\\b") << QStringLiteral("\\bwhile\\b") << QStringLiteral("\\breturn\\b")
                       << QStringLiteral("\\bimport\\b") << QStringLiteral("\\bfrom\\b") << QStringLiteral("\\bas\\b") << QStringLiteral("\\btry\\b")
                       << QStringLiteral("\\bexcept\\b") << QStringLiteral("\\bfinally\\b") << QStringLiteral("\\bwith\\b") << QStringLiteral("\\bassert\\b")
                       << QStringLiteral("\\bpass\\b") << QStringLiteral("\\bbreak\\b") << QStringLiteral("\\bcontinue\\b") << QStringLiteral("\\byield\\b");

        for (const QString &pattern : pythonKeywords) {
            HighlightingRule rule;
            rule.pattern = QRegularExpression(pattern);
            rule.format = keywordFormat;
            highlightingRules.append(rule);
        }
    }

    // C++ keywords
    else if (language == QStringLiteral("cpp")) {
        QStringList cppKeywords;
        cppKeywords << QStringLiteral("\\bvoid\\b") << QStringLiteral("\\bint\\b") << QStringLiteral("\\bfloat\\b") << QStringLiteral("\\bdouble\\b")
                    << QStringLiteral("\\bchar\\b") << QStringLiteral("\\bconst\\b") << QStringLiteral("\\bstatic\\b") << QStringLiteral("\\bclass\\b")
                    << QStringLiteral("\\bstruct\\b") << QStringLiteral("\\bif\\b") << QStringLiteral("\\belse\\b") << QStringLiteral("\\bfor\\b")
                    << QStringLiteral("\\bwhile\\b") << QStringLiteral("\\breturn\\b") << QStringLiteral("\\bnew\\b") << QStringLiteral("\\bdelete\\b")
                    << QStringLiteral("\\btemplate\\b") << QStringLiteral("\\btypename\\b") << QStringLiteral("\\bnamespace\\b") << QStringLiteral("\\bpublic\\b")
                    << QStringLiteral("\\bprivate\\b") << QStringLiteral("\\bprotected\\b");

        for (const QString &pattern : cppKeywords) {
            HighlightingRule rule;
            rule.pattern = QRegularExpression(pattern);
            rule.format = keywordFormat;
            highlightingRules.append(rule);
        }
    }

    // JavaScript/TypeScript keywords
    else if (language == QStringLiteral("javascript")) {
        QStringList jsKeywords;
        jsKeywords << QStringLiteral("\\bfunction\\b") << QStringLiteral("\\bvar\\b") << QStringLiteral("\\blet\\b") << QStringLiteral("\\bconst\\b")
                   << QStringLiteral("\\bif\\b") << QStringLiteral("\\belse\\b") << QStringLiteral("\\bfor\\b") << QStringLiteral("\\bwhile\\b")
                   << QStringLiteral("\\breturn\\b") << QStringLiteral("\\bclass\\b") << QStringLiteral("\\bimport\\b") << QStringLiteral("\\bexport\\b")
                   << QStringLiteral("\\basync\\b") << QStringLiteral("\\bawait\\b") << QStringLiteral("\\btry\\b") << QStringLiteral("\\bcatch\\b");

        for (const QString &pattern : jsKeywords) {
            HighlightingRule rule;
            rule.pattern = QRegularExpression(pattern);
            rule.format = keywordFormat;
            highlightingRules.append(rule);
        }
    }
    
    // Numbers
    HighlightingRule numberRule;
    numberRule.pattern = QRegularExpression(QStringLiteral("\\b[0-9]+\\b"));
    numberRule.format.setForeground(Qt::darkCyan);
    highlightingRules.append(numberRule);

    // Strings
    HighlightingRule stringRule;
    stringRule.pattern = QRegularExpression(QStringLiteral("\".*\""));
    stringRule.format = quotationFormat;
    highlightingRules.append(stringRule);
    
    // Comments
    commentStartExpression = QRegularExpression(QStringLiteral("/\\*"));
    commentEndExpression = QRegularExpression(QStringLiteral("\\*/"));
}

void SyntaxHighlighter::highlightBlock(const QString &text)
{
    // Apply highlighting rules
    for (const HighlightingRule &rule : highlightingRules) {
        QRegularExpressionMatchIterator matchIterator = rule.pattern.globalMatch(text);
        while (matchIterator.hasNext()) {
            QRegularExpressionMatch match = matchIterator.next();
            setFormat(match.capturedStart(), match.capturedLength(), rule.format);
        }
    }
    
    // Single line comments
    QRegularExpression singleLineCommentRegex(QStringLiteral("//.*"));
    QRegularExpressionMatchIterator matchIterator = singleLineCommentRegex.globalMatch(text);
    while (matchIterator.hasNext()) {
        QRegularExpressionMatch match = matchIterator.next();
        setFormat(match.capturedStart(), match.capturedLength(), singleLineCommentFormat);
    }
    
    // Multi-line comments
    setCurrentBlockState(0);
    
    int startIndex = 0;
    if (previousBlockState() != 1) {
        startIndex = text.indexOf(commentStartExpression);
    }
    
    while (startIndex >= 0) {
        QRegularExpressionMatch endMatch = commentEndExpression.match(text, startIndex);
        int endIndex = endMatch.capturedStart();
        int commentLength;
        
        if (endIndex == -1) {
            setCurrentBlockState(1);
            commentLength = text.length() - startIndex;
        } else {
            commentLength = endIndex - startIndex + endMatch.capturedLength();
        }
        
        setFormat(startIndex, commentLength, multiLineCommentFormat);
        startIndex = text.indexOf(commentStartExpression, startIndex + commentLength);
    }
}
