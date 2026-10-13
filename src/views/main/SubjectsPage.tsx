import { type CSSProperties, useEffect, useRef, useState } from "react";
import { NAME_MAX, PALETTE } from "../../features/subjects/palette";
import { moveItem, targetIndex } from "../../features/subjects/reorder";
import { api } from "../../shared/api";
import { play } from "../../shared/sfx";
import { useApp } from "../../shared/state";
import type { Subject } from "../../shared/types";
import { Button, Dot, TextButton } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";
import "./subjects.css";

const ROW_H = 54;

export function SubjectsPage() {
  const active = useApp((s) => s.subjects);
  const version = useApp((s) => s.dataVersion);
  const [archived, setArchived] = useState<Subject[]>([]);
  const [showArchived, setShowArchived] = useState(false);
  const [order, setOrder] = useState<Subject[]>(active);
  const [drag, setDrag] = useState<{ id: number; from: number; startY: number; dy: number } | null>(null);

  useEffect(() => setOrder(active), [active]);
  useEffect(() => {
    void api.subjects.list(true).then((all) => setArchived(all.filter((s) => s.archived)));
  }, [version]);

  const to = drag ? targetIndex(drag.from, order.length, drag.dy, ROW_H) : -1;
  const shift = (i: number) => {
    if (!drag || i === drag.from) return 0;
    if (drag.from < i && i <= to) return -ROW_H;
    if (to <= i && i < drag.from) return ROW_H;
    return 0;
  };

  const drop = (clientY: number) => {
    if (!drag) return;
    const to = targetIndex(drag.from, order.length, clientY - drag.startY, ROW_H);
    const next = moveItem(order, drag.from, to);
    setDrag(null);
    if (to === drag.from) return;
    play("toggle");
    setOrder(next);
    void attempt(api.subjects.reorder(next.map((s) => s.id)));
  };

  return (
    <div className="page subjects-page">
      <h1 className="page-title">科目</h1>
      <p className="page-hint">拖动左边的把手调整顺序。归档的科目不会出现在开始页，历史记录照常保留。</p>

      <div className="subject-list" style={{ height: order.length * ROW_H }}>
        {order.map((s, i) => (
          <SubjectRow
            key={s.id}
            subject={s}
            style={{
              top: i * ROW_H,
              transform: `translateY(${drag?.id === s.id ? drag.dy : shift(i)}px)`,
              transition: drag?.id === s.id ? "none" : undefined,
              zIndex: drag?.id === s.id ? 2 : undefined,
            }}
            dragging={drag?.id === s.id}
            onHandleDown={(e) => {
              e.currentTarget.setPointerCapture(e.pointerId);
              setDrag({ id: s.id, from: i, startY: e.clientY, dy: 0 });
            }}
            onHandleMove={(e) => {
              if (drag?.id === s.id) setDrag({ ...drag, dy: e.clientY - drag.startY });
            }}
            onHandleUp={drop}
          />
        ))}
      </div>

      <NewSubject />

      {archived.length > 0 && (
        <section className="archived">
          <TextButton className="link muted" onClick={() => setShowArchived(!showArchived)}>
            已归档 {archived.length} 个{showArchived ? "，收起" : ""}
          </TextButton>
          {showArchived &&
            archived.map((s) => (
              <div className="subject-row static" key={s.id}>
                <Dot color={s.color} />
                <span className="wk name dim">{s.name}</span>
                <TextButton onClick={() => void attempt(api.subjects.restore(s.id))}>恢复</TextButton>
              </div>
            ))}
        </section>
      )}
    </div>
  );
}

function SubjectRow({
  subject,
  style,
  dragging,
  onHandleDown,
  onHandleMove,
  onHandleUp,
}: {
  subject: Subject;
  style: CSSProperties;
  dragging: boolean;
  onHandleDown: (e: React.PointerEvent<HTMLElement>) => void;
  onHandleMove: (e: React.PointerEvent<HTMLElement>) => void;
  onHandleUp: (clientY: number) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [picking, setPicking] = useState(false);

  return (
    <div className={dragging ? "subject-row dragging" : "subject-row"} style={picking ? { ...style, zIndex: 3 } : style}>
      <span
        className="handle"
        role="button"
        aria-label={`拖动${subject.name}排序`}
        onPointerDown={onHandleDown}
        onPointerMove={onHandleMove}
        onPointerUp={(e) => onHandleUp(e.clientY)}
        onPointerCancel={(e) => onHandleUp(e.clientY)}
        onDragStart={(e) => e.preventDefault()}
      >
        ⋮⋮
      </span>
      <button
        type="button"
        className="swatch"
        aria-label="换颜色"
        style={{ "--c": subject.color } as CSSProperties}
        onClick={() => {
          play("tap");
          setPicking(!picking);
        }}
      />
      {editing ? (
        <NameInput
          initial={subject.name}
          onDone={(name) => {
            setEditing(false);
            if (name !== null && name !== subject.name) void attempt(api.subjects.update(subject.id, { name }));
          }}
        />
      ) : (
        <span
          className={subject.is_fallback ? "wk name" : "wk name editable"}
          title={subject.is_fallback ? undefined : "点一下改名"}
          onClick={() => !subject.is_fallback && setEditing(true)}
        >
          {subject.name}
        </span>
      )}
      {subject.is_fallback && <span className="tag">没选科目时记到这里</span>}
      <span className="spacer" />
      {!subject.is_fallback && (
        <TextButton
          className="link muted"
          cue="delete"
          onClick={() => void attempt(api.subjects.archive(subject.id))}
        >
          归档
        </TextButton>
      )}
      {picking && (
        <Palette
          value={subject.color}
          onPick={(color) => {
            setPicking(false);
            void attempt(api.subjects.update(subject.id, { color }));
          }}
        />
      )}
    </div>
  );
}

function NameInput({ initial, onDone }: { initial: string; onDone: (name: string | null) => void }) {
  const ref = useRef<HTMLInputElement>(null);
  const done = useRef(false);
  useEffect(() => ref.current?.select(), []);
  const finish = (name: string | null) => {
    if (done.current) return;
    done.current = true;
    onDone(name);
  };
  return (
    <input
      ref={ref}
      className="input small name-input"
      defaultValue={initial}
      maxLength={NAME_MAX}
      autoFocus
      onBlur={(e) => finish(e.currentTarget.value.trim())}
      onKeyDown={(e) => {
        if (e.key === "Enter") finish(e.currentTarget.value.trim());
        if (e.key === "Escape") finish(null);
      }}
    />
  );
}

function Palette({ value, onPick }: { value: string; onPick: (color: string) => void }) {
  return (
    <div className="palette">
      {PALETTE.map((c) => (
        <button
          type="button"
          key={c}
          aria-label={c}
          className={c.toLowerCase() === value.toLowerCase() ? "swatch on" : "swatch"}
          style={{ "--c": c } as CSSProperties}
          onClick={() => {
            play("tap");
            onPick(c);
          }}
        />
      ))}
    </div>
  );
}

function NewSubject() {
  const subjects = useApp((s) => s.subjects);
  const [name, setName] = useState("");
  const used = new Set(subjects.map((s) => s.color.toLowerCase()));
  const [color, setColor] = useState<string | null>(null);
  const picked = color ?? PALETTE.find((c) => !used.has(c.toLowerCase())) ?? PALETTE[0];
  const [picking, setPicking] = useState(false);

  const add = async () => {
    const created = await attempt(api.subjects.create(name.trim(), picked));
    if (created) {
      setName("");
      setColor(null);
    }
  };

  return (
    <div className="subject-new">
      <button
        type="button"
        className="swatch"
        aria-label="新科目的颜色"
        style={{ "--c": picked } as CSSProperties}
        onClick={() => {
          play("tap");
          setPicking(!picking);
        }}
      />
      <input
        className="input small"
        placeholder="新科目，比如“数据结构”"
        value={name}
        maxLength={NAME_MAX}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && name.trim() && void add()}
      />
      <Button size="small" disabled={!name.trim()} onClick={() => void add()}>
        添加
      </Button>
      {picking && (
        <Palette
          value={picked}
          onPick={(c) => {
            setColor(c);
            setPicking(false);
          }}
        />
      )}
    </div>
  );
}
